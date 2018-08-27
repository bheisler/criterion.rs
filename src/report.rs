use stats::bivariate::Data;
use stats::bivariate::regression::Slope;
use stats::univariate::outliers::tukey::LabeledSample;

use Estimate;
use estimate::{Distributions, Estimates, Statistic};
use format;
use stats::Distribution;
use stats::univariate::Sample;
use std::cell::Cell;
use std::cmp;
use std::collections::HashSet;
use std::fmt;
use std::io::Write;
use std::io::stdout;
use {PlotConfiguration, Plotting, Throughput};

const MAX_DIRECTORY_NAME_LEN: usize = 64;

pub(crate) struct ComparisonData {
    pub p_value: f64,
    pub t_distribution: Distribution<f64>,
    pub t_value: f64,
    pub relative_estimates: Estimates,
    pub relative_distributions: Distributions,
    pub significance_threshold: f64,
    pub noise_threshold: f64,
    pub base_iter_counts: Vec<f64>,
    pub base_sample_times: Vec<f64>,
    pub base_avg_times: Vec<f64>,
    pub base_estimates: Estimates,
}

pub(crate) struct MeasurementData<'a> {
    pub iter_counts: &'a Sample<f64>,
    pub sample_times: &'a Sample<f64>,
    pub avg_times: LabeledSample<'a, f64>,
    pub absolute_estimates: Estimates,
    pub distributions: Distributions,
    pub comparison: Option<ComparisonData>,
    pub throughput: Option<Throughput>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ValueType {
    Bytes,
    Elements,
    Value,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BenchmarkId {
    pub group_id: String,
    pub function_id: Option<String>,
    pub value_str: Option<String>,
    pub throughput: Option<Throughput>,
    full_id: String,
    directory_name: String,
}

fn make_filename_safe(string: &str) -> String {
    let mut string = string
        .to_owned()
        .replace("?", "_")
        .replace("\"", "_")
        .replace("/", "_")
        .replace("\\", "_")
        .replace("*", "_");

    // On Windows, file names are not case-sensitive, so lowercase everything.
    if cfg!(target_os = "windows") {
        string = string.to_lowercase();
    }

    // Truncate to last character boundary before max length...
    let mut boundary = cmp::min(MAX_DIRECTORY_NAME_LEN, string.len());
    while !string.is_char_boundary(boundary) {
        boundary -= 1;
    }
    string.truncate(boundary);

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
            (&Some(ref func), &Some(ref val)) => format!("{}/{}/{}", group_id, func, val),
            (&Some(ref func), &None) => format!("{}/{}", group_id, func),
            (&None, &Some(ref val)) => format!("{}/{}", group_id, val),
            (&None, &None) => group_id.clone(),
        };

        let directory_name = match (&function_id, &value_str) {
            (&Some(ref func), &Some(ref val)) => format!(
                "{}/{}/{}",
                make_filename_safe(&group_id),
                make_filename_safe(func),
                make_filename_safe(val)
            ),
            (&Some(ref func), &None) => format!(
                "{}/{}",
                make_filename_safe(&group_id),
                make_filename_safe(func)
            ),
            (&None, &Some(ref val)) => format!(
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
        }
    }

    pub fn id(&self) -> &str {
        &self.full_id
    }

    pub fn as_directory_name(&self) -> &str {
        &self.directory_name
    }

    pub fn as_number(&self) -> Option<f64> {
        match self.throughput {
            Some(Throughput::Bytes(n)) | Some(Throughput::Elements(n)) => Some(f64::from(n)),
            None => self.value_str
                .as_ref()
                .and_then(|string| string.parse::<f64>().ok()),
        }
    }

    pub fn value_type(&self) -> Option<ValueType> {
        match self.throughput {
            Some(Throughput::Bytes(_)) => Some(ValueType::Bytes),
            Some(Throughput::Elements(_)) => Some(ValueType::Elements),
            None => self.value_str
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
}
impl fmt::Display for BenchmarkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.id())
    }
}
impl fmt::Debug for BenchmarkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    pub output_directory: String,
    pub plotting: Plotting,
    pub plot_config: PlotConfiguration,
    pub test_mode: bool,
}

pub(crate) trait Report {
    fn benchmark_start(&self, _id: &BenchmarkId, _context: &ReportContext) {}
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
        _measurements: &MeasurementData,
    ) {
    }
    fn summarize(&self, _context: &ReportContext, _all_ids: &[BenchmarkId]) {}
    fn final_summary(&self, _context: &ReportContext) {}
}

pub(crate) struct Reports {
    reports: Vec<Box<Report>>,
}
impl Reports {
    pub fn new(reports: Vec<Box<Report>>) -> Reports {
        Reports { reports }
    }
}
impl Report for Reports {
    fn benchmark_start(&self, id: &BenchmarkId, context: &ReportContext) {
        for report in &self.reports {
            report.benchmark_start(id, context);
        }
    }

    fn warmup(&self, id: &BenchmarkId, context: &ReportContext, warmup_ns: f64) {
        for report in &self.reports {
            report.warmup(id, context, warmup_ns);
        }
    }

    fn terminated(&self, id: &BenchmarkId, context: &ReportContext) {
        for report in &self.reports {
            report.terminated(id, context);
        }
    }

    fn analysis(&self, id: &BenchmarkId, context: &ReportContext) {
        for report in &self.reports {
            report.analysis(id, context);
        }
    }

    fn measurement_start(
        &self,
        id: &BenchmarkId,
        context: &ReportContext,
        sample_count: u64,
        estimate_ns: f64,
        iter_count: u64,
    ) {
        for report in &self.reports {
            report.measurement_start(id, context, sample_count, estimate_ns, iter_count);
        }
    }

    fn measurement_complete(
        &self,
        id: &BenchmarkId,
        context: &ReportContext,
        measurements: &MeasurementData,
    ) {
        for report in &self.reports {
            report.measurement_complete(id, context, measurements);
        }
    }

    fn summarize(&self, context: &ReportContext, all_ids: &[BenchmarkId]) {
        for report in &self.reports {
            report.summarize(context, all_ids);
        }
    }

    fn final_summary(&self, context: &ReportContext) {
        for report in &self.reports {
            report.final_summary(context);
        }
    }
}

pub(crate) struct CliReport {
    pub enable_text_overwrite: bool,
    pub enable_text_coloring: bool,
    pub verbose: bool,

    last_line_len: Cell<usize>,
}
impl CliReport {
    pub fn new(
        enable_text_overwrite: bool,
        enable_text_coloring: bool,
        verbose: bool,
    ) -> CliReport {
        CliReport {
            enable_text_overwrite,
            enable_text_coloring,
            verbose,

            last_line_len: Cell::new(0),
        }
    }

    fn text_overwrite(&self) {
        if self.enable_text_overwrite {
            print!("\r");
            for _ in 0..self.last_line_len.get() {
                print!(" ");
            }
            print!("\r");
        }
    }

    //Passing a String is the common case here.
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    fn print_overwritable(&self, s: String) {
        if self.enable_text_overwrite {
            self.last_line_len.set(s.len());
            print!("{}", s);
            stdout().flush().unwrap();
        } else {
            println!("{}", s);
        }
    }

    fn green(&self, s: String) -> String {
        if self.enable_text_coloring {
            format!("\x1B[32m{}\x1B[39m", s)
        } else {
            s
        }
    }

    fn yellow(&self, s: String) -> String {
        if self.enable_text_coloring {
            format!("\x1B[33m{}\x1B[39m", s)
        } else {
            s
        }
    }

    fn red(&self, s: String) -> String {
        if self.enable_text_coloring {
            format!("\x1B[31m{}\x1B[39m", s)
        } else {
            s
        }
    }

    fn bold(&self, s: String) -> String {
        if self.enable_text_coloring {
            format!("\x1B[1m{}\x1B[22m", s)
        } else {
            s
        }
    }

    fn faint(&self, s: String) -> String {
        if self.enable_text_coloring {
            format!("\x1B[2m{}\x1B[22m", s)
        } else {
            s
        }
    }

    pub fn outliers(&self, sample: &LabeledSample<f64>) {
        let (los, lom, _, him, his) = sample.count();
        let noutliers = los + lom + him + his;
        let sample_size = sample.as_slice().len();

        if noutliers == 0 {
            return;
        }

        let percent = |n: usize| 100. * n as f64 / sample_size as f64;

        println!(
            "{}",
            self.yellow(format!(
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
    fn benchmark_start(&self, id: &BenchmarkId, ctx: &ReportContext) {
        if ctx.test_mode {
            println!("Testing {}", id);
        } else {
            self.print_overwritable(format!("Benchmarking {}", id));
        }
    }

    fn warmup(&self, id: &BenchmarkId, _: &ReportContext, warmup_ns: f64) {
        self.text_overwrite();
        self.print_overwritable(format!(
            "Benchmarking {}: Warming up for {}",
            id,
            format::time(warmup_ns)
        ));
    }

    fn terminated(&self, id: &BenchmarkId, ctx: &ReportContext) {
        if ctx.test_mode {
            println!("Success");
        } else {
            self.text_overwrite();
            println!("Benchmarking {}: Complete (Analysis Disabled)", id);
        }
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
        let iter_string = if self.verbose {
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

    fn measurement_complete(&self, id: &BenchmarkId, _: &ReportContext, meas: &MeasurementData) {
        self.text_overwrite();

        let slope_estimate = meas.absolute_estimates[&Statistic::Slope];

        {
            let mut id = id.id().to_owned();

            if id.len() > 23 {
                if id.len() > 80 {
                    id.truncate(77);
                    id.push_str("...");
                }
                println!("{}", self.green(id.clone()));
                id.clear();
            }
            let id_len = id.len();

            println!(
                "{}{}time:   [{} {} {}]",
                self.green(id),
                " ".repeat(24 - id_len),
                self.faint(format::time(slope_estimate.confidence_interval.lower_bound)),
                self.bold(format::time(slope_estimate.point_estimate)),
                self.faint(format::time(slope_estimate.confidence_interval.upper_bound))
            );
        }

        if let Some(ref throughput) = meas.throughput {
            println!(
                "{}thrpt:  [{} {} {}]",
                " ".repeat(24),
                self.faint(format::throughput(
                    throughput,
                    slope_estimate.confidence_interval.upper_bound
                )),
                self.bold(format::throughput(
                    throughput,
                    slope_estimate.point_estimate
                )),
                self.faint(format::throughput(
                    throughput,
                    slope_estimate.confidence_interval.lower_bound
                )),
            )
        }

        if let Some(ref comp) = meas.comparison {
            let different_mean = comp.p_value < comp.significance_threshold;
            let mean_est = comp.relative_estimates[&Statistic::Mean];
            let point_estimate = mean_est.point_estimate;
            let mut point_estimate_str = format::change(point_estimate, true);
            let explanation_str: String;

            if !different_mean {
                explanation_str = "No change in performance detected.".to_owned();
            } else {
                let comparison = compare_to_threshold(&mean_est, comp.noise_threshold);
                match comparison {
                    ComparisonResult::Improved => {
                        point_estimate_str = self.green(self.bold(point_estimate_str));
                        explanation_str =
                            format!("Performance has {}.", self.green("improved".to_owned()));
                    }
                    ComparisonResult::Regressed => {
                        point_estimate_str = self.red(self.bold(point_estimate_str));
                        explanation_str =
                            format!("Performance has {}.", self.red("regressed".to_owned()));
                    }
                    ComparisonResult::NonSignificant => {
                        explanation_str = "Change within noise threshold.".to_owned();
                    }
                }
            }

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
            println!("{}{}", " ".repeat(24), explanation_str);
        }

        self.outliers(&meas.avg_times);

        if self.verbose {
            let data = Data::new(meas.iter_counts.as_slice(), meas.sample_times.as_slice());
            let slope_estimate = &meas.absolute_estimates[&Statistic::Slope];

            fn format_short_estimate(estimate: &Estimate) -> String {
                format!(
                    "[{} {}]",
                    format::time(estimate.confidence_interval.lower_bound),
                    format::time(estimate.confidence_interval.upper_bound)
                )
            }

            println!(
                "{:<7}{} {:<15}[{:0.7} {:0.7}]",
                "slope",
                format_short_estimate(slope_estimate),
                "R^2",
                Slope(slope_estimate.confidence_interval.lower_bound).r_squared(data),
                Slope(slope_estimate.confidence_interval.upper_bound).r_squared(data),
            );
            println!(
                "{:<7}{} {:<15}{}",
                "mean",
                format_short_estimate(&meas.absolute_estimates[&Statistic::Mean]),
                "std. dev.",
                format_short_estimate(&meas.absolute_estimates[&Statistic::StdDev]),
            );
            println!(
                "{:<7}{} {:<15}{}",
                "median",
                format_short_estimate(&meas.absolute_estimates[&Statistic::Median]),
                "med. abs. dev.",
                format_short_estimate(&meas.absolute_estimates[&Statistic::MedianAbsDev]),
            );
        }
    }
}

enum ComparisonResult {
    Improved,
    Regressed,
    NonSignificant,
}

fn compare_to_threshold(estimate: &Estimate, noise: f64) -> ComparisonResult {
    let ci = estimate.confidence_interval;
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

        new_id = existing_id.clone();
        new_id.ensure_directory_name_unique(&directories);
        assert_eq!("group/function/value_3", new_id.as_directory_name());
        directories.insert(new_id.as_directory_name().to_owned());
    }
}
