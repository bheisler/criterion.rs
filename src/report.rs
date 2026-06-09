#[cfg(feature = "csv_output")]
use crate::csv_report::FileCsvReport;
use crate::stats::bivariate::regression::Slope;
use crate::stats::univariate::outliers::tukey::LabeledSample;
use crate::{html::Html, stats::bivariate::Data};

use crate::estimate::{
    ChangeDistributions, ChangeEstimates, Distributions, Estimate, Estimates, Statistic,
};
use crate::format;
use crate::fs;
use crate::measurement::ValueFormatter;
use crate::stats::univariate::Sample;
use crate::stats::Distribution;
use crate::{PlotConfiguration, Throughput};
use anes::{Attribute, ClearLine, Color, ResetAttributes, SetAttribute, SetForegroundColor};
use serde::{Deserialize, Serialize};
use std::cmp::{self, Ordering};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::stderr;
use std::io::Write;
use std::path::{Path, PathBuf};

const MAX_DIRECTORY_NAME_LEN: usize = 64;
const MAX_TITLE_LEN: usize = 100;

pub(crate) struct ComparisonData {
    pub p_value: f64,
    pub t_distribution: Distribution<f64>,
    pub t_value: f64,
    pub relative_estimates: ChangeEstimates,
    pub relative_distributions: ChangeDistributions,
    pub significance_threshold: f64,
    pub noise_threshold: f64,
    pub base_iter_counts: Vec<f64>,
    pub base_sample_times: Vec<f64>,
    pub base_avg_times: Vec<f64>,
    pub base_estimates: Estimates,
}

pub(crate) struct MeasurementData<'a> {
    pub data: Data<'a, f64, f64>,
    pub avg_times: LabeledSample<'a, f64>,
    pub absolute_estimates: Estimates,
    pub distributions: Distributions,
    pub comparison: Option<ComparisonData>,
    pub throughput: Option<Throughput>,
}
impl<'a> MeasurementData<'a> {
    pub fn iter_counts(&self) -> &Sample<f64> {
        self.data.x()
    }

    #[cfg(feature = "csv_output")]
    pub fn sample_times(&self) -> &Sample<f64> {
        self.data.y()
    }
}

#[derive(Debug)]
pub(crate) struct ComparisonCell {
    pub name: String,
    pub rank: usize,
    pub ratio: f64,
    pub formatted_value: String,
    pub throughput_value: Option<String>,
    pub is_best: bool,
    pub delta_to_next: Option<f64>,
    pub change: Option<f64>,
    pub change_positive: Option<bool>,
    pub score_delta: Option<f64>,
}

#[derive(Debug)]
pub(crate) struct ComparisonRow {
    pub label: &'static str,
    pub cells: Vec<ComparisonCell>,
}

const COMPARISON_STATS: &[(Statistic, &str)] = &[(Statistic::Typical, "time")];
const SCORE_EPS: f64 = 0.0005;

fn ordinal(n: usize) -> String {
    let rem100 = n % 100;
    let suffix = if (11..=13).contains(&rem100) {
        "th"
    } else {
        match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };
    format!("{n}{suffix}")
}

pub(crate) fn load_estimates_for_ids<'a>(
    output_directory: &Path,
    ids: &[&'a BenchmarkId],
) -> Vec<(&'a BenchmarkId, Estimates)> {
    ids.iter()
        .filter_map(|id| {
            let entry = output_directory
                .join(id.as_directory_name())
                .join("new")
                .join("estimates.json");
            fs::load(&entry).ok().map(|estimates| (*id, estimates))
        })
        .collect()
}

pub(crate) fn load_change_for_ids(
    output_directory: &Path,
    ids: &[&BenchmarkId],
) -> HashMap<String, f64> {
    ids.iter()
        .filter_map(|id| {
            let entry = output_directory
                .join(id.as_directory_name())
                .join("change")
                .join("estimates.json");
            fs::load::<ChangeEstimates, _>(&entry)
                .ok()
                .map(|estimates| {
                    (
                        id.as_directory_name().to_owned(),
                        estimates.mean.point_estimate,
                    )
                })
        })
        .collect()
}

fn shared_throughput_and_times(
    entries: &[(&BenchmarkId, Estimates)],
) -> Option<(Throughput, Vec<f64>)> {
    let mut iter = entries.iter();
    let first = iter.next()?;
    let throughput = first.0.throughput.clone()?;

    if iter.all(|(id, _)| id.throughput.as_ref() == Some(&throughput)) {
        let times = entries
            .iter()
            .map(|(_, est)| est.typical().point_estimate)
            .collect();
        Some((throughput, times))
    } else {
        None
    }
}

fn throughput_rate(throughput: &Throughput, time_ns: f64) -> f64 {
    let iters_per_second = 1e9 / time_ns;
    match throughput {
        Throughput::Bytes(b)
        | Throughput::BytesDecimal(b)
        | Throughput::Elements(b)
        | Throughput::Bits(b) => *b as f64 * iters_per_second,
    }
}

fn format_values_with_unit(values: &[f64], formatter: &dyn ValueFormatter) -> Option<Vec<String>> {
    if values.is_empty() {
        return None;
    }
    let typical = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !typical.is_finite() {
        return None;
    }

    let mut scaled = values.to_vec();
    let unit = formatter.scale_values(typical, &mut scaled);

    Some(
        scaled
            .into_iter()
            .map(|v| format!("{:>6} {}", format::short(v), unit))
            .collect(),
    )
}

fn format_throughput_values(
    throughput: &Throughput,
    times: &[f64],
    formatter: &dyn ValueFormatter,
) -> Option<Vec<String>> {
    if times.is_empty() {
        return None;
    }
    let typical = times.iter().copied().fold(f64::INFINITY, f64::min);
    if !typical.is_finite() || typical <= 0.0 {
        return None;
    }

    let mut time_values = times.to_vec();
    let unit = formatter.scale_throughputs(typical, throughput, &mut time_values);

    Some(
        time_values
            .into_iter()
            .map(|v| format!("{:>6} {}", format::short(v), unit))
            .collect(),
    )
}

pub(crate) fn build_comparison_rows(
    entries: &[(&BenchmarkId, Estimates)],
    formatter: &dyn ValueFormatter,
    change: Option<&HashMap<String, f64>>,
) -> Vec<ComparisonRow> {
    let mut rows = Vec::new();

    for (stat, label) in COMPARISON_STATS {
        let mut values = Vec::with_capacity(entries.len());
        let mut base_values: Vec<(usize, f64)> = Vec::with_capacity(entries.len());
        for (_, est) in entries {
            if let Some(value) = est.get(*stat) {
                values.push(value.point_estimate);
            } else {
                values.clear();
                break;
            }
        }
        if values.is_empty() {
            continue;
        }

        for (idx, (id, _)) in entries.iter().enumerate() {
            if let Some(change_ratio) = change
                .and_then(|map: &HashMap<_, _>| map.get(id.as_directory_name()))
                .copied()
            {
                let current = values[idx];
                let base = current / (1.0 + change_ratio);
                if base.is_finite() && base > 0.0 {
                    base_values.push((idx, base));
                }
            }
        }

        let mut base_ranks: HashMap<usize, usize> = HashMap::new();
        base_values.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Greater));
        for (rank, (idx, _)) in base_values.into_iter().enumerate() {
            base_ranks.insert(idx, rank + 1);
        }
        let mut base_ratio_map = HashMap::new();
        if !base_ranks.is_empty() {
            let mut base_values_only = Vec::new();
            for (idx, _) in base_ranks.iter() {
                if let Some(change_ratio) = change
                    .and_then(|map: &HashMap<_, _>| map.get(entries[*idx].0.as_directory_name()))
                {
                    let current = values[*idx];
                    let base_value = current / (1.0 + change_ratio);
                    base_values_only.push((*idx, base_value));
                }
            }
            if !base_values_only.is_empty() {
                let base_best_val = base_values_only
                    .iter()
                    .map(|(_, v)| *v)
                    .fold(f64::INFINITY, f64::min);
                if base_best_val.is_finite() && base_best_val > 0.0 {
                    for (idx, base_val) in base_values_only {
                        base_ratio_map.insert(idx, base_best_val / base_val);
                    }
                }
            }
        }

        let best = values.iter().copied().fold(f64::INFINITY, f64::min);
        if !best.is_finite() || best <= 0.0 {
            continue;
        }

        let ratios: Vec<f64> = values.iter().map(|v| best / *v).collect();
        let best_ratio = ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let formatted_values = match format_values_with_unit(&values, formatter) {
            Some(values) => values,
            None => continue,
        };

        let mut cells: Vec<_> = entries
            .iter()
            .enumerate()
            .zip(ratios.iter())
            .zip(formatted_values.into_iter())
            .map(|(((idx, (id, _)), &ratio), formatted_value)| (idx, id, ratio, formatted_value))
            .collect();
        cells.sort_by(|(_, _, a, _), (_, _, b, _)| {
            b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut cells: Vec<ComparisonCell> = cells
            .into_iter()
            .enumerate()
            .map(|(idx, (original_idx, id, ratio, formatted_value))| {
                let change_ratio = change
                    .and_then(|map: &HashMap<_, _>| map.get(id.as_directory_name()))
                    .copied();
                let score_delta = base_ratio_map
                    .get(&original_idx)
                    .map(|base_ratio| ratio - base_ratio);
                ComparisonCell {
                    name: id.as_title().to_owned(),
                    rank: idx + 1,
                    ratio,
                    formatted_value,
                    throughput_value: None,
                    is_best: (ratio - best_ratio).abs() < f64::EPSILON,
                    delta_to_next: None,
                    change: change_ratio,
                    change_positive: change_ratio.map(|c| c > 0.0),
                    score_delta,
                }
            })
            .collect();

        for idx in 0..cells.len().saturating_sub(1) {
            let next_ratio = cells[idx + 1].ratio;
            if next_ratio > 0.0 {
                let savings = (cells[idx].ratio / next_ratio - 1.0) * 100.0;
                cells[idx].delta_to_next = Some(savings);
            }
        }

        rows.push(ComparisonRow { label, cells });
    }

    if let Some((throughput, times)) = shared_throughput_and_times(entries) {
        let mut rates = Vec::with_capacity(times.len());
        for time in &times {
            if *time <= 0.0 {
                rates.clear();
                break;
            }
            rates.push(throughput_rate(&throughput, *time));
        }

        if !rates.is_empty() {
            let mut base_rates = Vec::with_capacity(rates.len());
            for (idx, (id, _)) in entries.iter().enumerate() {
                if let (Some(change_ratio), Some(time)) = (
                    change
                        .and_then(|map: &HashMap<_, _>| map.get(id.as_directory_name()))
                        .copied(),
                    times.get(idx),
                ) {
                    let base_time = *time / (1.0 + change_ratio);
                    if base_time.is_finite() && base_time > 0.0 {
                        base_rates.push((idx, throughput_rate(&throughput, base_time)));
                    }
                }
            }

            let mut base_ranks: HashMap<usize, usize> = HashMap::new();
            base_rates.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(Ordering::Less));
            for (rank, (idx, _)) in base_rates.iter().enumerate() {
                base_ranks.insert(*idx, rank + 1);
            }
            let mut base_ratio_map = HashMap::new();
            if !base_ranks.is_empty() {
                let base_best_val = base_rates
                    .iter()
                    .map(|(_, v)| *v)
                    .fold(f64::NEG_INFINITY, f64::max);
                if base_best_val.is_finite() && base_best_val > 0.0 {
                    for (idx, base_val) in base_rates.iter() {
                        base_ratio_map.insert(*idx, base_val / base_best_val);
                    }
                }
            }

            let best = rates.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            if best.is_finite() && best > 0.0 {
                let ratios: Vec<f64> = rates.iter().map(|r| r / best).collect();
                let best_ratio = ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                if let Some(formatted_values) =
                    format_throughput_values(&throughput, &times, formatter)
                {
                    let mut cells: Vec<_> = entries
                        .iter()
                        .enumerate()
                        .zip(ratios.iter())
                        .zip(formatted_values)
                        .map(|(((idx, (id, _)), &ratio), formatted_value)| {
                            (idx, id, ratio, formatted_value)
                        })
                        .collect();
                    cells.sort_by(|(_, _, a, _), (_, _, b, _)| {
                        b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    let mut cells: Vec<ComparisonCell> = cells
                        .into_iter()
                        .enumerate()
                        .map(|(idx, (original_idx, id, ratio, formatted_value))| {
                            let change_ratio = change
                                .and_then(|map: &HashMap<_, _>| map.get(id.as_directory_name()))
                                .copied();
                            let score_delta = base_ratio_map
                                .get(&original_idx)
                                .map(|base_ratio| ratio - base_ratio);
                            ComparisonCell {
                                name: id.as_title().to_owned(),
                                rank: idx + 1,
                                ratio,
                                formatted_value,
                                throughput_value: None,
                                is_best: (ratio - best_ratio).abs() < f64::EPSILON,
                                delta_to_next: None,
                                change: change_ratio,
                                change_positive: change_ratio.map(|c| c > 0.0),
                                score_delta,
                            }
                        })
                        .collect();

                    for idx in 0..cells.len().saturating_sub(1) {
                        let next_ratio = cells[idx + 1].ratio;
                        if next_ratio > 0.0 && cells[idx].ratio > 0.0 {
                            let savings = (cells[idx].ratio / next_ratio - 1.0) * 100.0;
                            cells[idx].delta_to_next = Some(savings);
                        }
                    }

                    rows.push(ComparisonRow {
                        label: "throughput",
                        cells,
                    });
                }
            }
        }
    }

    rows
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ValueType {
    Bytes,
    Elements,
    Bits,
    Value,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BenchmarkId {
    pub group_id: String,
    pub function_id: Option<String>,
    pub value_str: Option<String>,
    pub throughput: Option<Throughput>,
    full_id: String,
    directory_name: String,
    title: String,
}

fn truncate_to_character_boundary(s: &mut String, max_len: usize) {
    let mut boundary = cmp::min(max_len, s.len());
    while !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    s.truncate(boundary);
}

pub fn make_filename_safe(string: &str) -> String {
    let mut string = string.replace(
        &['?', '"', '/', '\\', '*', '<', '>', ':', '|', '^'][..],
        "_",
    );

    // Truncate to last character boundary before max length...
    truncate_to_character_boundary(&mut string, MAX_DIRECTORY_NAME_LEN);

    if cfg!(target_os = "windows") {
        {
            string = string
                // On Windows, spaces in the end of the filename are ignored and will be trimmed.
                //
                // Without trimming ourselves, creating a directory `dir ` will silently create
                // `dir` instead, but then operations on files like `dir /file` will fail.
                //
                // Also note that it's important to do this *after* trimming to MAX_DIRECTORY_NAME_LEN,
                // otherwise it can trim again to a name with a trailing space.
                .trim_end()
                // On Windows, file names are not case-sensitive, so lowercase everything.
                .to_lowercase();
        }
    }

    string
}

impl BenchmarkId {
    pub fn new(
        group_id: String,
        function_id: Option<String>,
        value_str: Option<String>,
        throughput: Option<Throughput>,
    ) -> BenchmarkId {
        let full_id = match (&function_id, &value_str) {
            (Some(func), Some(val)) => format!("{}/{}/{}", group_id, func, val),
            (Some(func), &None) => format!("{}/{}", group_id, func),
            (&None, Some(val)) => format!("{}/{}", group_id, val),
            (&None, &None) => group_id.clone(),
        };

        let mut title = full_id.clone();
        truncate_to_character_boundary(&mut title, MAX_TITLE_LEN);
        if title != full_id {
            title.push_str("...");
        }

        let directory_name = match (&function_id, &value_str) {
            (Some(func), Some(val)) => format!(
                "{}/{}/{}",
                make_filename_safe(&group_id),
                make_filename_safe(func),
                make_filename_safe(val)
            ),
            (Some(func), &None) => format!(
                "{}/{}",
                make_filename_safe(&group_id),
                make_filename_safe(func)
            ),
            (&None, Some(val)) => format!(
                "{}/{}",
                make_filename_safe(&group_id),
                make_filename_safe(val)
            ),
            (&None, &None) => make_filename_safe(&group_id),
        };

        BenchmarkId {
            group_id,
            function_id,
            value_str,
            throughput,
            full_id,
            directory_name,
            title,
        }
    }

    pub fn id(&self) -> &str {
        &self.full_id
    }

    pub fn as_title(&self) -> &str {
        &self.title
    }

    pub fn as_directory_name(&self) -> &str {
        &self.directory_name
    }

    pub fn as_number(&self) -> Option<f64> {
        match self.throughput {
            Some(Throughput::Bytes(n))
            | Some(Throughput::Elements(n))
            | Some(Throughput::BytesDecimal(n))
            | Some(Throughput::Bits(n)) => Some(n as f64),
            None => self
                .value_str
                .as_ref()
                .and_then(|string| string.parse::<f64>().ok()),
        }
    }

    pub fn value_type(&self) -> Option<ValueType> {
        match self.throughput {
            Some(Throughput::Bytes(_)) => Some(ValueType::Bytes),
            Some(Throughput::BytesDecimal(_)) => Some(ValueType::Bytes),
            Some(Throughput::Elements(_)) => Some(ValueType::Elements),
            Some(Throughput::Bits(_)) => Some(ValueType::Bits),
            None => self
                .value_str
                .as_ref()
                .and_then(|string| string.parse::<f64>().ok())
                .map(|_| ValueType::Value),
        }
    }

    pub fn ensure_directory_name_unique(&mut self, existing_directories: &HashSet<String>) {
        if !existing_directories.contains(self.as_directory_name()) {
            return;
        }

        let mut counter = 2;
        loop {
            let new_dir_name = format!("{}_{}", self.as_directory_name(), counter);
            if !existing_directories.contains(&new_dir_name) {
                self.directory_name = new_dir_name;
                return;
            }
            counter += 1;
        }
    }

    pub fn ensure_title_unique(&mut self, existing_titles: &HashSet<String>) {
        if !existing_titles.contains(self.as_title()) {
            return;
        }

        let mut counter = 2;
        loop {
            let new_title = format!("{} #{}", self.as_title(), counter);
            if !existing_titles.contains(&new_title) {
                self.title = new_title;
                return;
            }
            counter += 1;
        }
    }
}
impl fmt::Display for BenchmarkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_title())
    }
}
impl fmt::Debug for BenchmarkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format_opt(opt: &Option<String>) -> String {
            match *opt {
                Some(ref string) => format!("\"{}\"", string),
                None => "None".to_owned(),
            }
        }

        write!(
            f,
            "BenchmarkId {{ group_id: \"{}\", function_id: {}, value_str: {}, throughput: {:?} }}",
            self.group_id,
            format_opt(&self.function_id),
            format_opt(&self.value_str),
            self.throughput,
        )
    }
}

pub struct ReportContext {
    pub output_directory: PathBuf,
    pub plot_config: PlotConfiguration,
    pub comparison: bool,
}
impl ReportContext {
    pub fn report_path<P: AsRef<Path> + ?Sized>(&self, id: &BenchmarkId, file_name: &P) -> PathBuf {
        let mut path = self.output_directory.clone();
        path.push(id.as_directory_name());
        path.push("report");
        path.push(file_name);
        path
    }
}

pub(crate) trait Report {
    fn test_start(&self, _id: &BenchmarkId, _context: &ReportContext) {}
    fn test_pass(&self, _id: &BenchmarkId, _context: &ReportContext) {}

    fn benchmark_start(&self, _id: &BenchmarkId, _context: &ReportContext) {}
    fn profile(&self, _id: &BenchmarkId, _context: &ReportContext, _profile_ns: f64) {}
    fn warmup(&self, _id: &BenchmarkId, _context: &ReportContext, _warmup_ns: f64) {}
    fn terminated(&self, _id: &BenchmarkId, _context: &ReportContext) {}
    fn analysis(&self, _id: &BenchmarkId, _context: &ReportContext) {}
    fn measurement_start(
        &self,
        _id: &BenchmarkId,
        _context: &ReportContext,
        _sample_count: u64,
        _estimate_ns: f64,
        _iter_count: u64,
    ) {
    }
    fn measurement_complete(
        &self,
        _id: &BenchmarkId,
        _context: &ReportContext,
        _measurements: &MeasurementData<'_>,
        _formatter: &dyn ValueFormatter,
    ) {
    }
    fn summarize(
        &self,
        _context: &ReportContext,
        _all_ids: &[BenchmarkId],
        _formatter: &dyn ValueFormatter,
    ) {
    }
    fn final_summary(&self, _context: &ReportContext) {}
    fn group_separator(&self) {}
}

pub(crate) struct Reports {
    pub(crate) cli_enabled: bool,
    pub(crate) cli: CliReport,
    pub(crate) bencher_enabled: bool,
    pub(crate) bencher: BencherReport,
    pub(crate) csv_enabled: bool,
    pub(crate) html: Option<Html>,
}
macro_rules! reports_impl {
    (fn $name:ident(&self, $($argn:ident: $argt:ty),*)) => {
        fn $name(&self, $($argn: $argt),* ) {
            if self.cli_enabled {
                self.cli.$name($($argn),*);
            }
            if self.bencher_enabled {
                self.bencher.$name($($argn),*);
            }
            #[cfg(feature = "csv_output")]
            if self.csv_enabled {
                FileCsvReport.$name($($argn),*);
            }
            if let Some(reporter) = &self.html {
                reporter.$name($($argn),*);
            }
        }
    };
}

impl Report for Reports {
    reports_impl!(fn test_start(&self, id: &BenchmarkId, context: &ReportContext));
    reports_impl!(fn test_pass(&self, id: &BenchmarkId, context: &ReportContext));
    reports_impl!(fn benchmark_start(&self, id: &BenchmarkId, context: &ReportContext));
    reports_impl!(fn profile(&self, id: &BenchmarkId, context: &ReportContext, profile_ns: f64));
    reports_impl!(fn warmup(&self, id: &BenchmarkId, context: &ReportContext, warmup_ns: f64));
    reports_impl!(fn terminated(&self, id: &BenchmarkId, context: &ReportContext));
    reports_impl!(fn analysis(&self, id: &BenchmarkId, context: &ReportContext));
    reports_impl!(fn measurement_start(
        &self,
        id: &BenchmarkId,
        context: &ReportContext,
        sample_count: u64,
        estimate_ns: f64,
        iter_count: u64
    ));
    reports_impl!(
    fn measurement_complete(
        &self,
        id: &BenchmarkId,
        context: &ReportContext,
        measurements: &MeasurementData<'_>,
        formatter: &dyn ValueFormatter
    ));
    reports_impl!(
    fn summarize(
        &self,
        context: &ReportContext,
        all_ids: &[BenchmarkId],
        formatter: &dyn ValueFormatter
    ));

    reports_impl!(fn final_summary(&self, context: &ReportContext));
    reports_impl!(fn group_separator(&self, ));
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum CliVerbosity {
    Quiet,
    Normal,
    Verbose,
}

pub(crate) struct CliReport {
    pub enable_text_overwrite: bool,
    pub enable_text_coloring: bool,
    pub verbosity: CliVerbosity,
}
impl CliReport {
    pub fn new(
        enable_text_overwrite: bool,
        enable_text_coloring: bool,
        verbosity: CliVerbosity,
    ) -> CliReport {
        CliReport {
            enable_text_overwrite,
            enable_text_coloring,
            verbosity,
        }
    }

    fn text_overwrite(&self) {
        if self.enable_text_overwrite {
            eprint!("\r{}", ClearLine::All);
        }
    }

    // Passing a String is the common case here.
    #[allow(clippy::needless_pass_by_value)]
    fn print_overwritable(&self, s: String) {
        if self.enable_text_overwrite {
            eprint!("{}", s);
            stderr().flush().unwrap();
        } else {
            eprintln!("{}", s);
        }
    }

    fn with_color(&self, color: Color, s: &str) -> String {
        if self.enable_text_coloring {
            format!("{}{}{}", SetForegroundColor(color), s, ResetAttributes)
        } else {
            String::from(s)
        }
    }

    fn green(&self, s: &str) -> String {
        self.with_color(Color::DarkGreen, s)
    }

    fn yellow(&self, s: &str) -> String {
        self.with_color(Color::DarkYellow, s)
    }

    fn red(&self, s: &str) -> String {
        self.with_color(Color::DarkRed, s)
    }

    fn bold(&self, s: String) -> String {
        if self.enable_text_coloring {
            format!("{}{}{}", SetAttribute(Attribute::Bold), s, ResetAttributes)
        } else {
            s
        }
    }

    fn faint(&self, s: String) -> String {
        if self.enable_text_coloring {
            format!("{}{}{}", SetAttribute(Attribute::Faint), s, ResetAttributes)
        } else {
            s
        }
    }

    pub fn outliers(&self, sample: &LabeledSample<'_, f64>) {
        let (los, lom, _, him, his) = sample.count();
        let noutliers = los + lom + him + his;
        let sample_size = sample.len();

        if noutliers == 0 {
            return;
        }

        let percent = |n: usize| 100. * n as f64 / sample_size as f64;

        println!(
            "{}",
            self.yellow(&format!(
                "Found {} outliers among {} measurements ({:.2}%)",
                noutliers,
                sample_size,
                percent(noutliers)
            ))
        );

        let print = |n, label| {
            if n != 0 {
                println!("  {} ({:.2}%) {}", n, percent(n), label);
            }
        };

        print(los, "low severe");
        print(lom, "low mild");
        print(him, "high mild");
        print(his, "high severe");
    }
}
impl Report for CliReport {
    fn test_start(&self, id: &BenchmarkId, _: &ReportContext) {
        println!("Testing {}", id);
    }
    fn test_pass(&self, _: &BenchmarkId, _: &ReportContext) {
        println!("Success");
    }

    fn benchmark_start(&self, id: &BenchmarkId, _: &ReportContext) {
        self.print_overwritable(format!("Benchmarking {}", id));
    }

    fn profile(&self, id: &BenchmarkId, _: &ReportContext, warmup_ns: f64) {
        self.text_overwrite();
        self.print_overwritable(format!(
            "Benchmarking {}: Profiling for {}",
            id,
            format::time(warmup_ns)
        ));
    }

    fn warmup(&self, id: &BenchmarkId, _: &ReportContext, warmup_ns: f64) {
        self.text_overwrite();
        self.print_overwritable(format!(
            "Benchmarking {}: Warming up for {}",
            id,
            format::time(warmup_ns)
        ));
    }

    fn terminated(&self, id: &BenchmarkId, _: &ReportContext) {
        self.text_overwrite();
        println!("Benchmarking {}: Complete (Analysis Disabled)", id);
    }

    fn analysis(&self, id: &BenchmarkId, _: &ReportContext) {
        self.text_overwrite();
        self.print_overwritable(format!("Benchmarking {}: Analyzing", id));
    }

    fn measurement_start(
        &self,
        id: &BenchmarkId,
        _: &ReportContext,
        sample_count: u64,
        estimate_ns: f64,
        iter_count: u64,
    ) {
        self.text_overwrite();
        let iter_string = if matches!(self.verbosity, CliVerbosity::Verbose) {
            format!("{} iterations", iter_count)
        } else {
            format::iter_count(iter_count)
        };

        self.print_overwritable(format!(
            "Benchmarking {}: Collecting {} samples in estimated {} ({})",
            id,
            sample_count,
            format::time(estimate_ns),
            iter_string
        ));
    }

    fn measurement_complete(
        &self,
        id: &BenchmarkId,
        _: &ReportContext,
        meas: &MeasurementData<'_>,
        formatter: &dyn ValueFormatter,
    ) {
        self.text_overwrite();

        let typical_estimate = &meas.absolute_estimates.typical();

        {
            let mut id = id.as_title().to_owned();

            if id.len() > 23 {
                println!("{}", self.green(&id));
                id.clear();
            }
            let id_len = id.len();

            println!(
                "{}{}time:   [{} {} {}]",
                self.green(&id),
                " ".repeat(24 - id_len),
                self.faint(
                    formatter.format_value(typical_estimate.confidence_interval.lower_bound)
                ),
                self.bold(formatter.format_value(typical_estimate.point_estimate)),
                self.faint(
                    formatter.format_value(typical_estimate.confidence_interval.upper_bound)
                )
            );
        }

        if let Some(ref throughput) = meas.throughput {
            println!(
                "{}thrpt:  [{} {} {}]",
                " ".repeat(24),
                self.faint(formatter.format_throughput(
                    throughput,
                    typical_estimate.confidence_interval.upper_bound
                )),
                self.bold(formatter.format_throughput(throughput, typical_estimate.point_estimate)),
                self.faint(formatter.format_throughput(
                    throughput,
                    typical_estimate.confidence_interval.lower_bound
                )),
            );
        }

        if !matches!(self.verbosity, CliVerbosity::Quiet) {
            if let Some(ref comp) = meas.comparison {
                let different_mean = comp.p_value < comp.significance_threshold;
                let mean_est = &comp.relative_estimates.mean;
                let point_estimate = mean_est.point_estimate;
                let mut point_estimate_str = format::change(point_estimate, true);
                // The change in throughput is related to the change in timing. Reducing the timing by
                // 50% increases the throughput by 100%.
                let to_thrpt_estimate = |ratio: f64| 1.0 / (1.0 + ratio) - 1.0;
                let mut thrpt_point_estimate_str =
                    format::change(to_thrpt_estimate(point_estimate), true);
                let explanation_str: String;

                if !different_mean {
                    explanation_str = "No change in performance detected.".to_owned();
                } else {
                    let comparison = compare_to_threshold(mean_est, comp.noise_threshold);
                    match comparison {
                        ComparisonResult::Improved => {
                            point_estimate_str = self.green(&self.bold(point_estimate_str));
                            thrpt_point_estimate_str =
                                self.green(&self.bold(thrpt_point_estimate_str));
                            explanation_str =
                                format!("Performance has {}.", self.green("improved"));
                        }
                        ComparisonResult::Regressed => {
                            point_estimate_str = self.red(&self.bold(point_estimate_str));
                            thrpt_point_estimate_str =
                                self.red(&self.bold(thrpt_point_estimate_str));
                            explanation_str = format!("Performance has {}.", self.red("regressed"));
                        }
                        ComparisonResult::NonSignificant => {
                            explanation_str = "Change within noise threshold.".to_owned();
                        }
                    }
                }

                if meas.throughput.is_some() {
                    println!("{}change:", " ".repeat(17));

                    println!(
                        "{}time:   [{} {} {}] (p = {:.2} {} {:.2})",
                        " ".repeat(24),
                        self.faint(format::change(
                            mean_est.confidence_interval.lower_bound,
                            true
                        )),
                        point_estimate_str,
                        self.faint(format::change(
                            mean_est.confidence_interval.upper_bound,
                            true
                        )),
                        comp.p_value,
                        if different_mean { "<" } else { ">" },
                        comp.significance_threshold
                    );
                    println!(
                        "{}thrpt:  [{} {} {}]",
                        " ".repeat(24),
                        self.faint(format::change(
                            to_thrpt_estimate(mean_est.confidence_interval.upper_bound),
                            true
                        )),
                        thrpt_point_estimate_str,
                        self.faint(format::change(
                            to_thrpt_estimate(mean_est.confidence_interval.lower_bound),
                            true
                        )),
                    );
                } else {
                    println!(
                        "{}change: [{} {} {}] (p = {:.2} {} {:.2})",
                        " ".repeat(24),
                        self.faint(format::change(
                            mean_est.confidence_interval.lower_bound,
                            true
                        )),
                        point_estimate_str,
                        self.faint(format::change(
                            mean_est.confidence_interval.upper_bound,
                            true
                        )),
                        comp.p_value,
                        if different_mean { "<" } else { ">" },
                        comp.significance_threshold
                    );
                }

                println!("{}{}", " ".repeat(24), explanation_str);
            }
        }

        if !matches!(self.verbosity, CliVerbosity::Quiet) {
            self.outliers(&meas.avg_times);
        }

        if matches!(self.verbosity, CliVerbosity::Verbose) {
            let format_short_estimate = |estimate: &Estimate| -> String {
                format!(
                    "[{} {}]",
                    formatter.format_value(estimate.confidence_interval.lower_bound),
                    formatter.format_value(estimate.confidence_interval.upper_bound)
                )
            };

            let data = &meas.data;
            if let Some(slope_estimate) = meas.absolute_estimates.slope.as_ref() {
                println!(
                    "{:<7}{} {:<15}[{:0.7} {:0.7}]",
                    "slope",
                    format_short_estimate(slope_estimate),
                    "R^2",
                    Slope(slope_estimate.confidence_interval.lower_bound).r_squared(data),
                    Slope(slope_estimate.confidence_interval.upper_bound).r_squared(data),
                );
            }
            println!(
                "{:<7}{} {:<15}{}",
                "mean",
                format_short_estimate(&meas.absolute_estimates.mean),
                "std. dev.",
                format_short_estimate(&meas.absolute_estimates.std_dev),
            );
            println!(
                "{:<7}{} {:<15}{}",
                "median",
                format_short_estimate(&meas.absolute_estimates.median),
                "med. abs. dev.",
                format_short_estimate(&meas.absolute_estimates.median_abs_dev),
            );
        }
    }

    fn summarize(
        &self,
        context: &ReportContext,
        all_ids: &[BenchmarkId],
        formatter: &dyn ValueFormatter,
    ) {
        if !context.comparison || all_ids.len() < 2 || matches!(self.verbosity, CliVerbosity::Quiet)
        {
            return;
        }

        let available_ids: Vec<_> = all_ids
            .iter()
            .filter(|id| {
                let id_dir = context.output_directory.join(id.as_directory_name());
                fs::is_dir(&id_dir)
            })
            .collect();
        if available_ids.len() < 2 {
            return;
        }

        // Check if benchmarks are parameterized (have both function_id and value_str).
        // If so, produce per-value comparison tables so that implementations are compared
        // at each parameter value separately rather than as one flat list.
        let has_subgroups = available_ids
            .iter()
            .all(|id| id.function_id.is_some() && id.value_str.is_some());

        if has_subgroups {
            let mut value_strs: Vec<&str> = available_ids
                .iter()
                .filter_map(|id| id.value_str.as_deref())
                .collect();
            let all_numeric = value_strs.iter().all(|s| s.parse::<f64>().is_ok());
            if all_numeric {
                value_strs.sort_by(|a, b| {
                    a.parse::<f64>()
                        .unwrap()
                        .partial_cmp(&b.parse::<f64>().unwrap())
                        .unwrap_or(Ordering::Equal)
                });
            } else {
                value_strs.sort();
            }
            value_strs.dedup();

            let group_id = &available_ids[0].group_id;
            self.text_overwrite();
            println!(
                "Comparison for group '{}' (by parameter):",
                group_id
            );
            println!("  Higher is better; best performer is 1.00 (typical).");

            for value_str in &value_strs {
                let sub_ids: Vec<_> = available_ids
                    .iter()
                    .filter(|id| id.value_str.as_deref() == Some(*value_str))
                    .copied()
                    .collect();
                if sub_ids.len() < 2 {
                    continue;
                }

                let entries =
                    load_estimates_for_ids(&context.output_directory, &sub_ids);
                let change =
                    load_change_for_ids(&context.output_directory, &sub_ids);
                if entries.len() < 2 {
                    continue;
                }

                let rows = build_comparison_rows(&entries, formatter, Some(&change));
                if rows.is_empty() {
                    continue;
                }

                println!();
                println!("  n = {}:", value_str);
                self.print_comparison_rows(&rows);
            }
        } else {
            let entries = load_estimates_for_ids(&context.output_directory, &available_ids);
            let change = load_change_for_ids(&context.output_directory, &available_ids);
            if entries.len() < 2 {
                return;
            }

            let rows = build_comparison_rows(&entries, formatter, Some(&change));
            if rows.is_empty() {
                return;
            }

            self.text_overwrite();
            println!("Comparison for group '{}':", entries[0].0.group_id);
            println!("  Higher is better; best performer is 1.00 (typical).");

            self.print_comparison_rows(&rows);
        }
    }

    fn group_separator(&self) {
        println!();
    }
}

impl CliReport {
    fn print_comparison_rows(&self, rows: &[ComparisonRow]) {
        let show_labels = rows.len() > 1;
        let max_label_len = rows
            .iter()
            .flat_map(|r| r.cells.iter())
            .map(|c| ordinal(c.rank).len() + 1 + c.name.len())
            .max()
            .unwrap_or(0);

        for row in rows {
            if show_labels {
                println!("    {}:", row.label);
            }
            for (idx, cell) in row.cells.iter().enumerate() {
                let ratio_str = format!("{:.3}", cell.ratio);
                let ratio_str = if cell.is_best {
                    self.green(&self.bold(ratio_str))
                } else if cell.ratio < 0.999 {
                    self.red(&ratio_str)
                } else {
                    ratio_str
                };

                let value_str = if cell.is_best {
                    self.green(&self.bold(cell.formatted_value.clone()))
                } else {
                    self.red(&cell.formatted_value)
                };

                let mut value_with_delta = value_str.clone();
                if let Some(thr) = cell.throughput_value.as_ref() {
                    let trimmed = thr.trim_start();
                    value_with_delta.push_str(&format!(", {}", trimmed));
                }

                let label_plain_len = ordinal(cell.rank).len() + 1 + cell.name.len();
                let padding = max_label_len.saturating_sub(label_plain_len);
                let label_colored = format!("{} {}", self.bold(ordinal(cell.rank)), cell.name);

                let change_str = cell
                    .change
                    .map(|c| {
                        let change_text = format::change(c, true);
                        let score_delta = cell.score_delta.map(|d| {
                            if d.abs() < SCORE_EPS {
                                "0.000".to_owned()
                            } else if d > 0.0 {
                                self.green(&self.bold(format!("{:+.3}", d)))
                            } else {
                                self.red(&self.bold(format!("{:+.3}", d)))
                            }
                        });
                        let change_segment = if c < 0.0 {
                            self.green(&self.bold(change_text))
                        } else if c > 0.0 {
                            self.red(&self.bold(change_text))
                        } else {
                            change_text
                        };
                        let score_segment = score_delta.unwrap_or_else(|| "0.000".to_owned());

                        format!(", change: [{}, {}]", score_segment, change_segment)
                    })
                    .unwrap_or_default();

                let mut rank_parts = Vec::new();
                if idx > 0 {
                    let prev_ratio = row.cells[idx - 1].ratio;
                    if cell.ratio > 0.0 && prev_ratio.is_finite() {
                        let slower_pct = (prev_ratio / cell.ratio - 1.0) * 100.0;
                        if slower_pct.is_finite() {
                            let slower_text = if slower_pct > 200.0 {
                                format!("{:.2}x", 1.0 + slower_pct / 100.0)
                            } else {
                                format!("{:.1}%", slower_pct)
                            };
                            let slower_str = self.red(&slower_text);
                            rank_parts.push(format!(
                                "{} slower than {}",
                                slower_str,
                                self.bold(ordinal(cell.rank - 1))
                            ));
                        }
                    }
                }
                if let Some(delta) = cell.delta_to_next {
                    let delta_text = if delta > 200.0 {
                        format!("{:.2}x", 1.0 + delta / 100.0)
                    } else {
                        format!("{:.1}%", delta)
                    };
                    let delta_pct = self.green(&delta_text);
                    rank_parts.push(format!(
                        "{} faster than {}",
                        delta_pct,
                        self.bold(ordinal(cell.rank + 1))
                    ));
                }
                let rank_str = if rank_parts.is_empty() {
                    String::new()
                } else {
                    format!(", rank: [{}]", rank_parts.join(", "))
                };

                println!(
                    "      {}:{pad}{ratio} ({value}){change}{rank}",
                    label_colored,
                    pad = " ".repeat(padding + 1),
                    ratio = ratio_str,
                    value = value_with_delta,
                    change = change_str,
                    rank = rank_str,
                );
            }
        }
    }
}

pub struct BencherReport;
impl Report for BencherReport {
    fn measurement_start(
        &self,
        id: &BenchmarkId,
        _context: &ReportContext,
        _sample_count: u64,
        _estimate_ns: f64,
        _iter_count: u64,
    ) {
        print!("test {} ... ", id);
    }

    fn measurement_complete(
        &self,
        _id: &BenchmarkId,
        _: &ReportContext,
        meas: &MeasurementData<'_>,
        formatter: &dyn ValueFormatter,
    ) {
        let mut values = [
            meas.absolute_estimates.median.point_estimate,
            meas.absolute_estimates.std_dev.point_estimate,
        ];
        let unit = formatter.scale_for_machines(&mut values);

        println!(
            "bench: {:>11} {}/iter (+/- {})",
            format::integer(values[0]),
            unit,
            format::integer(values[1])
        );
    }

    fn group_separator(&self) {
        println!();
    }
}

enum ComparisonResult {
    Improved,
    Regressed,
    NonSignificant,
}

fn compare_to_threshold(estimate: &Estimate, noise: f64) -> ComparisonResult {
    let ci = &estimate.confidence_interval;
    let lb = ci.lower_bound;
    let ub = ci.upper_bound;

    if lb < -noise && ub < -noise {
        ComparisonResult::Improved
    } else if lb > noise && ub > noise {
        ComparisonResult::Regressed
    } else {
        ComparisonResult::NonSignificant
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::estimate::ConfidenceInterval;
    use crate::measurement::{Measurement, WallTime};

    #[test]
    fn test_make_filename_safe_replaces_characters() {
        let input = "?/\\*\"";
        let safe = make_filename_safe(input);
        assert_eq!("_____", &safe);
    }

    #[test]
    fn test_make_filename_safe_truncates_long_strings() {
        let input = "this is a very long string. it is too long to be safe as a directory name, and so it needs to be truncated. what a long string this is.";
        let safe = make_filename_safe(input);
        assert!(input.len() > MAX_DIRECTORY_NAME_LEN);
        assert_eq!(&input[0..MAX_DIRECTORY_NAME_LEN], &safe);
    }

    #[test]
    fn test_make_filename_safe_respects_character_boundaries() {
        let input = "✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓";
        let safe = make_filename_safe(input);
        assert!(safe.len() < MAX_DIRECTORY_NAME_LEN);
    }

    #[test]
    fn test_benchmark_id_make_directory_name_unique() {
        let existing_id = BenchmarkId::new(
            "group".to_owned(),
            Some("function".to_owned()),
            Some("value".to_owned()),
            None,
        );
        let mut directories = HashSet::new();
        directories.insert(existing_id.as_directory_name().to_owned());

        let mut new_id = existing_id.clone();
        new_id.ensure_directory_name_unique(&directories);
        assert_eq!("group/function/value_2", new_id.as_directory_name());
        directories.insert(new_id.as_directory_name().to_owned());

        new_id = existing_id;
        new_id.ensure_directory_name_unique(&directories);
        assert_eq!("group/function/value_3", new_id.as_directory_name());
        directories.insert(new_id.as_directory_name().to_owned());
    }
    #[test]
    fn test_benchmark_id_make_long_directory_name_unique() {
        let long_name = (0..MAX_DIRECTORY_NAME_LEN).map(|_| 'a').collect::<String>();
        let existing_id = BenchmarkId::new(long_name, None, None, None);
        let mut directories = HashSet::new();
        directories.insert(existing_id.as_directory_name().to_owned());

        let mut new_id = existing_id.clone();
        new_id.ensure_directory_name_unique(&directories);
        assert_ne!(existing_id.as_directory_name(), new_id.as_directory_name());
    }

    fn estimate_with_value(value: f64) -> Estimate {
        Estimate {
            confidence_interval: ConfidenceInterval {
                confidence_level: 0.95,
                lower_bound: value,
                upper_bound: value,
            },
            point_estimate: value,
            standard_error: 0.0,
        }
    }

    fn estimates_for_point(value: f64) -> Estimates {
        Estimates {
            mean: estimate_with_value(value),
            median: estimate_with_value(value),
            median_abs_dev: estimate_with_value(0.0),
            slope: None,
            std_dev: estimate_with_value(0.0),
        }
    }

    #[test]
    fn build_comparison_rows_prefers_lower_times() {
        let formatter = WallTime.formatter();
        let fast_id = BenchmarkId::new("group".to_owned(), Some("fast".to_owned()), None, None);
        let slow_id = BenchmarkId::new("group".to_owned(), Some("slow".to_owned()), None, None);

        let rows = build_comparison_rows(
            &[
                (&fast_id, estimates_for_point(10.0)),
                (&slow_id, estimates_for_point(20.0)),
            ],
            formatter,
            None,
        );

        let typical = rows.iter().find(|r| r.label == "typical").unwrap();
        assert_eq!(typical.cells.len(), 2);
        assert!(typical.cells[0].is_best);
        assert!(typical.cells[0].ratio > typical.cells[1].ratio);
    }

    #[test]
    fn build_comparison_rows_include_throughput() {
        let formatter = WallTime.formatter();
        let fast_id = BenchmarkId::new(
            "group".to_owned(),
            Some("fast".to_owned()),
            None,
            Some(Throughput::Bytes(100)),
        );
        let slow_id = BenchmarkId::new(
            "group".to_owned(),
            Some("slow".to_owned()),
            None,
            Some(Throughput::Bytes(100)),
        );

        let rows = build_comparison_rows(
            &[
                (&fast_id, estimates_for_point(10.0)),
                (&slow_id, estimates_for_point(20.0)),
            ],
            formatter,
            None,
        );

        let throughput_row = rows
            .iter()
            .find(|r| r.label == "throughput")
            .expect("throughput row missing");
        assert_eq!(throughput_row.cells.len(), 2);
        assert!(throughput_row.cells[0].is_best);
        assert!(throughput_row.cells[0].ratio > throughput_row.cells[1].ratio);
    }
}
