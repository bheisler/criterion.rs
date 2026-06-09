use crate::report::{
    build_comparison_rows, load_change_for_ids, load_estimates_for_ids, make_filename_safe,
    BenchmarkId, ComparisonRow, MeasurementData, Report, ReportContext,
};
use crate::stats::bivariate::regression::Slope;

use crate::estimate::Estimate;
use crate::format;
use crate::fs;
use crate::measurement::ValueFormatter;
use crate::plot::{PlotContext, PlotData, Plotter};
use crate::SavedSample;
use criterion_plot::Size;
use serde::Serialize;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use tinytemplate::TinyTemplate;

const THUMBNAIL_SIZE: Option<Size> = Some(Size(450, 300));

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

fn debug_context<S: Serialize>(path: &Path, context: &S) {
    if crate::debug_enabled() {
        let mut context_path = PathBuf::from(path);
        context_path.set_extension("json");
        println!("Writing report context to {:?}", context_path);
        let result = fs::save(context, &context_path);
        if let Err(e) = result {
            error!("Failed to write report context debug output: {}", e);
        }
    }
}

#[derive(Serialize)]
struct Context {
    title: String,
    confidence: String,

    thumbnail_width: usize,
    thumbnail_height: usize,

    slope: Option<ConfidenceInterval>,
    r2: ConfidenceInterval,
    mean: ConfidenceInterval,
    std_dev: ConfidenceInterval,
    median: ConfidenceInterval,
    mad: ConfidenceInterval,
    throughput: Option<ConfidenceInterval>,

    additional_plots: Vec<Plot>,

    comparison: Option<Comparison>,
}

#[derive(Serialize)]
struct IndividualBenchmark {
    name: String,
    path: String,
    regression_exists: bool,
}
impl IndividualBenchmark {
    fn from_id(
        output_directory: &Path,
        path_prefix: &str,
        id: &BenchmarkId,
    ) -> IndividualBenchmark {
        let mut regression_path = PathBuf::from(output_directory);
        regression_path.push(id.as_directory_name());
        regression_path.push("report");
        regression_path.push("regression.svg");

        IndividualBenchmark {
            name: id.as_title().to_owned(),
            path: format!("{}/{}", path_prefix, id.as_directory_name()),
            regression_exists: regression_path.is_file(),
        }
    }
}

#[derive(Serialize)]
struct SummaryContext {
    group_id: String,

    thumbnail_width: usize,
    thumbnail_height: usize,

    violin_plot: Option<String>,
    line_chart: Option<String>,

    comparison: Option<Vec<SummaryComparisonMetric>>,
    benchmarks: Vec<IndividualBenchmark>,
}

#[derive(Serialize)]
struct SummaryComparisonMetric {
    name: String,
    values: Vec<SummaryComparisonValue>,
}

#[derive(Serialize)]
struct SummaryComparisonValue {
    name: String,
    rank: usize,
    rank_ordinal: String,
    rank_class: Option<String>,
    ratio: String,
    value: String,
    is_best: bool,
    delta_to_next: Option<String>,
    change: Option<String>,
    change_class: Option<String>,
    rank_delta: Option<String>,
    score_delta: Option<String>,
    score_class: Option<String>,
    next_rank_ordinal: Option<String>,
}

impl From<ComparisonRow> for SummaryComparisonMetric {
    fn from(row: ComparisonRow) -> Self {
        fn format_rank_delta(value: f64) -> String {
            if value > 200.0 {
                format!("{:.2}x", 1.0 + value / 100.0)
            } else {
                format!("{:.1}%", value)
            }
        }

        SummaryComparisonMetric {
            name: row.label.to_owned(),
            values: row
                .cells
                .into_iter()
                .map(|cell| SummaryComparisonValue {
                    name: cell.name,
                    rank: cell.rank,
                    rank_ordinal: ordinal(cell.rank),
                    rank_class: None,
                    ratio: format!("{:.3}", cell.ratio),
                    value: cell.formatted_value,
                    is_best: cell.is_best,
                    delta_to_next: cell.delta_to_next.map(format_rank_delta),
                    change: cell.change.map(|c| format::change(c, true)),
                    change_class: match cell.change_positive {
                        Some(true) => Some("regressed".to_owned()),
                        Some(false) => Some("improved".to_owned()),
                        None => None,
                    },
                    rank_delta: None,
                    score_delta: cell.score_delta.map(|d| {
                        if d.abs() < 0.0005 {
                            "0.000".to_owned()
                        } else {
                            format!("{:+.3}", d)
                        }
                    }),
                    score_class: cell.score_delta.map(|d| {
                        if d.abs() < 0.0005 {
                            "".to_owned()
                        } else if d > 0.0 {
                            "improved".to_owned()
                        } else {
                            "regressed".to_owned()
                        }
                    }),
                    next_rank_ordinal: cell.delta_to_next.map(|_| ordinal(cell.rank + 1)),
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct ConfidenceInterval {
    lower: String,
    upper: String,
    point: String,
}

#[derive(Serialize)]
struct Plot {
    name: String,
    url: String,
}
impl Plot {
    fn new(name: &str, url: &str) -> Plot {
        Plot {
            name: name.to_owned(),
            url: url.to_owned(),
        }
    }
}

#[derive(Serialize)]
struct Comparison {
    p_value: String,
    inequality: String,
    significance_level: String,
    explanation: String,

    change: ConfidenceInterval,
    thrpt_change: Option<ConfidenceInterval>,
    additional_plots: Vec<Plot>,
}

fn if_exists(output_directory: &Path, path: &Path) -> Option<String> {
    let report_path = path.join("report/index.html");
    if PathBuf::from(output_directory).join(&report_path).is_file() {
        Some(report_path.to_string_lossy().to_string())
    } else {
        None
    }
}
#[derive(Serialize, Debug)]
struct ReportLink<'a> {
    name: &'a str,
    path: Option<String>,
}
impl<'a> ReportLink<'a> {
    // TODO: Would be nice if I didn't have to keep making these components filename-safe.
    fn group(output_directory: &Path, group_id: &'a str) -> ReportLink<'a> {
        let path = PathBuf::from(make_filename_safe(group_id));

        ReportLink {
            name: group_id,
            path: if_exists(output_directory, &path),
        }
    }

    fn function(output_directory: &Path, group_id: &str, function_id: &'a str) -> ReportLink<'a> {
        let mut path = PathBuf::from(make_filename_safe(group_id));
        path.push(make_filename_safe(function_id));

        ReportLink {
            name: function_id,
            path: if_exists(output_directory, &path),
        }
    }

    fn value(output_directory: &Path, group_id: &str, value_str: &'a str) -> ReportLink<'a> {
        let mut path = PathBuf::from(make_filename_safe(group_id));
        path.push(make_filename_safe(value_str));

        ReportLink {
            name: value_str,
            path: if_exists(output_directory, &path),
        }
    }

    fn individual(output_directory: &Path, id: &'a BenchmarkId) -> ReportLink<'a> {
        let path = PathBuf::from(id.as_directory_name());
        ReportLink {
            name: id.as_title(),
            path: if_exists(output_directory, &path),
        }
    }
}

#[derive(Serialize)]
struct BenchmarkValueGroup<'a> {
    value: Option<ReportLink<'a>>,
    benchmarks: Vec<ReportLink<'a>>,
}

#[derive(Serialize)]
struct BenchmarkGroup<'a> {
    group_report: ReportLink<'a>,

    function_ids: Option<Vec<ReportLink<'a>>>,
    values: Option<Vec<ReportLink<'a>>>,

    individual_links: Vec<BenchmarkValueGroup<'a>>,
}
impl<'a> BenchmarkGroup<'a> {
    fn new(output_directory: &Path, ids: &[&'a BenchmarkId]) -> BenchmarkGroup<'a> {
        let group_id = &ids[0].group_id;
        let group_report = ReportLink::group(output_directory, group_id);

        let mut function_ids = Vec::with_capacity(ids.len());
        let mut values = Vec::with_capacity(ids.len());
        let mut individual_links = HashMap::with_capacity(ids.len());

        for id in ids.iter() {
            let function_id = id.function_id.as_deref();
            let value = id.value_str.as_deref();

            let individual_link = ReportLink::individual(output_directory, id);

            function_ids.push(function_id);
            values.push(value);

            individual_links.insert((function_id, value), individual_link);
        }

        fn parse_opt(os: &Option<&str>) -> Option<f64> {
            os.and_then(|s| s.parse::<f64>().ok())
        }

        // If all of the value strings can be parsed into a number, sort/dedupe
        // numerically. Otherwise sort lexicographically.
        if values.iter().all(|os| parse_opt(os).is_some()) {
            values.sort_unstable_by(|v1, v2| {
                let num1 = parse_opt(v1);
                let num2 = parse_opt(v2);

                num1.partial_cmp(&num2).unwrap_or(Ordering::Less)
            });
            values.dedup_by_key(|os| parse_opt(os).unwrap());
        } else {
            values.sort_unstable();
            values.dedup();
        }

        // Sort and dedupe functions by name.
        function_ids.sort_unstable();
        function_ids.dedup();

        let mut value_groups = Vec::with_capacity(values.len());
        for value in values.iter() {
            let row = function_ids
                .iter()
                .filter_map(|f| individual_links.remove(&(*f, *value)))
                .collect::<Vec<_>>();
            value_groups.push(BenchmarkValueGroup {
                value: value.map(|s| ReportLink::value(output_directory, group_id, s)),
                benchmarks: row,
            });
        }

        let function_ids = function_ids
            .into_iter()
            .map(|os| os.map(|s| ReportLink::function(output_directory, group_id, s)))
            .collect::<Option<Vec<_>>>();
        let values = values
            .into_iter()
            .map(|os| os.map(|s| ReportLink::value(output_directory, group_id, s)))
            .collect::<Option<Vec<_>>>();

        BenchmarkGroup {
            group_report,
            function_ids,
            values,
            individual_links: value_groups,
        }
    }
}

#[derive(Serialize)]
struct IndexContext<'a> {
    groups: Vec<BenchmarkGroup<'a>>,
}

pub struct Html {
    templates: TinyTemplate<'static>,
    plotter: RefCell<Box<dyn Plotter>>,
}
impl Html {
    pub(crate) fn new(plotter: Box<dyn Plotter>) -> Html {
        let mut templates = TinyTemplate::new();
        templates
            .add_template("report_link", include_str!("report_link.html.tt"))
            .expect("Unable to parse report_link template.");
        templates
            .add_template("index", include_str!("index.html.tt"))
            .expect("Unable to parse index template.");
        templates
            .add_template("benchmark_report", include_str!("benchmark_report.html.tt"))
            .expect("Unable to parse benchmark_report template");
        templates
            .add_template("summary_report", include_str!("summary_report.html.tt"))
            .expect("Unable to parse summary_report template");

        let plotter = RefCell::new(plotter);
        Html { templates, plotter }
    }
}
impl Report for Html {
    fn measurement_complete(
        &self,
        id: &BenchmarkId,
        report_context: &ReportContext,
        measurements: &MeasurementData<'_>,
        formatter: &dyn ValueFormatter,
    ) {
        try_else_return!({
            let mut report_dir = report_context.output_directory.clone();
            report_dir.push(id.as_directory_name());
            report_dir.push("report");
            fs::mkdirp(&report_dir)
        });

        let typical_estimate = &measurements.absolute_estimates.typical();

        let time_interval = |est: &Estimate| -> ConfidenceInterval {
            ConfidenceInterval {
                lower: formatter.format_value(est.confidence_interval.lower_bound),
                point: formatter.format_value(est.point_estimate),
                upper: formatter.format_value(est.confidence_interval.upper_bound),
            }
        };

        let data = measurements.data;

        elapsed! {
            "Generating plots",
            self.generate_plots(id, report_context, formatter, measurements)
        }

        let mut additional_plots = vec![
            Plot::new("Typical", "typical.svg"),
            Plot::new("Mean", "mean.svg"),
            Plot::new("Std. Dev.", "SD.svg"),
            Plot::new("Median", "median.svg"),
            Plot::new("MAD", "MAD.svg"),
        ];
        if measurements.absolute_estimates.slope.is_some() {
            additional_plots.push(Plot::new("Slope", "slope.svg"));
        }

        let throughput = measurements
            .throughput
            .as_ref()
            .map(|thr| ConfidenceInterval {
                lower: formatter
                    .format_throughput(thr, typical_estimate.confidence_interval.upper_bound),
                upper: formatter
                    .format_throughput(thr, typical_estimate.confidence_interval.lower_bound),
                point: formatter.format_throughput(thr, typical_estimate.point_estimate),
            });

        let context = Context {
            title: id.as_title().to_owned(),
            confidence: format!(
                "{:.2}",
                typical_estimate.confidence_interval.confidence_level
            ),

            thumbnail_width: THUMBNAIL_SIZE.unwrap().0,
            thumbnail_height: THUMBNAIL_SIZE.unwrap().1,

            slope: measurements
                .absolute_estimates
                .slope
                .as_ref()
                .map(time_interval),
            mean: time_interval(&measurements.absolute_estimates.mean),
            median: time_interval(&measurements.absolute_estimates.median),
            mad: time_interval(&measurements.absolute_estimates.median_abs_dev),
            std_dev: time_interval(&measurements.absolute_estimates.std_dev),
            throughput,

            r2: ConfidenceInterval {
                lower: format!(
                    "{:0.7}",
                    Slope(typical_estimate.confidence_interval.lower_bound).r_squared(&data)
                ),
                upper: format!(
                    "{:0.7}",
                    Slope(typical_estimate.confidence_interval.upper_bound).r_squared(&data)
                ),
                point: format!(
                    "{:0.7}",
                    Slope(typical_estimate.point_estimate).r_squared(&data)
                ),
            },

            additional_plots,

            comparison: self.comparison(measurements),
        };

        let mut report_path = report_context.output_directory.clone();
        report_path.push(id.as_directory_name());
        report_path.push("report");
        report_path.push("index.html");
        debug_context(&report_path, &context);

        let text = self
            .templates
            .render("benchmark_report", &context)
            .expect("Failed to render benchmark report template");
        try_else_return!(fs::save_string(&text, &report_path));
    }

    fn summarize(
        &self,
        context: &ReportContext,
        all_ids: &[BenchmarkId],
        formatter: &dyn ValueFormatter,
    ) {
        let all_ids = all_ids
            .iter()
            .filter(|id| {
                let id_dir = context.output_directory.join(id.as_directory_name());
                fs::is_dir(&id_dir)
            })
            .collect::<Vec<_>>();
        if all_ids.is_empty() {
            return;
        }

        let group_id = all_ids[0].group_id.clone();

        let data = self.load_summary_data(&context.output_directory, &all_ids);

        let mut function_ids = BTreeSet::new();
        let mut value_strs = Vec::with_capacity(all_ids.len());
        for id in all_ids {
            if let Some(ref function_id) = id.function_id {
                function_ids.insert(function_id);
            }
            if let Some(ref value_str) = id.value_str {
                value_strs.push(value_str);
            }
        }

        fn try_parse(s: &str) -> Option<f64> {
            s.parse::<f64>().ok()
        }

        // If all of the value strings can be parsed into a number, sort/dedupe
        // numerically. Otherwise sort lexicographically.
        if value_strs.iter().all(|os| try_parse(os).is_some()) {
            value_strs.sort_unstable_by(|v1, v2| {
                let num1 = try_parse(v1);
                let num2 = try_parse(v2);

                num1.partial_cmp(&num2).unwrap_or(Ordering::Less)
            });
            value_strs.dedup_by_key(|os| try_parse(os).unwrap());
        } else {
            value_strs.sort_unstable();
            value_strs.dedup();
        }

        for function_id in function_ids {
            let samples_with_function: Vec<_> = data
                .iter()
                .by_ref()
                .filter(|&&(id, _)| id.function_id.as_ref() == Some(function_id))
                .collect();

            if samples_with_function.len() > 1 {
                let subgroup_id =
                    BenchmarkId::new(group_id.clone(), Some(function_id.clone()), None, None);

                self.generate_summary(
                    &subgroup_id,
                    &samples_with_function,
                    context,
                    formatter,
                    false,
                    false,
                );
            }
        }

        for value_str in value_strs {
            let samples_with_value: Vec<_> = data
                .iter()
                .by_ref()
                .filter(|&&(id, _)| id.value_str.as_ref() == Some(value_str))
                .collect();

            if samples_with_value.len() > 1 {
                let subgroup_id =
                    BenchmarkId::new(group_id.clone(), None, Some(value_str.clone()), None);

                // Enable comparison tables for per-value sub-groups when comparison mode
                // is active, so each parameter value gets its own ranked comparison.
                let enable_comparison = context.comparison;
                self.generate_summary(
                    &subgroup_id,
                    &samples_with_value,
                    context,
                    formatter,
                    false,
                    enable_comparison,
                );
            }
        }

        let mut all_data = data.iter().by_ref().collect::<Vec<_>>();
        // First sort the ids/data by value.
        // If all of the value strings can be parsed into a number, sort/dedupe
        // numerically. Otherwise sort lexicographically.
        let all_values_numeric = all_data
            .iter()
            .all(|(id, _)| id.value_str.as_deref().and_then(try_parse).is_some());
        if all_values_numeric {
            all_data.sort_unstable_by(|(a, _), (b, _)| {
                let num1 = a.value_str.as_deref().and_then(try_parse);
                let num2 = b.value_str.as_deref().and_then(try_parse);

                num1.partial_cmp(&num2).unwrap_or(Ordering::Less)
            });
        } else {
            all_data.sort_unstable_by_key(|(id, _)| id.value_str.as_ref());
        }
        // Next, sort the ids/data by function name. This results in a sorting priority of
        // function name, then value. This one has to be a stable sort.
        all_data.sort_by_key(|(id, _)| id.function_id.as_ref());

        self.generate_summary(
            &BenchmarkId::new(group_id, None, None, None),
            &all_data,
            context,
            formatter,
            true,
            true,
        );
        self.plotter.borrow_mut().wait();
    }

    fn final_summary(&self, report_context: &ReportContext) {
        let output_directory = &report_context.output_directory;
        if !fs::is_dir(&output_directory) {
            return;
        }

        let mut found_ids = try_else_return!(fs::list_existing_benchmarks(&output_directory));
        found_ids.sort_unstable_by_key(|id| id.id().to_owned());

        // Group IDs by group id
        let mut id_groups: HashMap<&str, Vec<&BenchmarkId>> = HashMap::new();
        for id in found_ids.iter() {
            id_groups.entry(&id.group_id).or_default().push(id);
        }

        let mut groups = id_groups
            .into_values()
            .map(|group| BenchmarkGroup::new(output_directory, &group))
            .collect::<Vec<BenchmarkGroup<'_>>>();
        groups.sort_unstable_by_key(|g| g.group_report.name);

        try_else_return!(fs::mkdirp(&output_directory.join("report")));

        let report_path = output_directory.join("report").join("index.html");

        let context = IndexContext { groups };

        debug_context(&report_path, &context);

        let text = self
            .templates
            .render("index", &context)
            .expect("Failed to render index template");
        try_else_return!(fs::save_string(&text, &report_path,));
    }
}
impl Html {
    fn comparison(&self, measurements: &MeasurementData<'_>) -> Option<Comparison> {
        if let Some(ref comp) = measurements.comparison {
            let different_mean = comp.p_value < comp.significance_threshold;
            let mean_est = &comp.relative_estimates.mean;
            let explanation_str: String;

            if !different_mean {
                explanation_str = "No change in performance detected.".to_owned();
            } else {
                let comparison = compare_to_threshold(mean_est, comp.noise_threshold);
                match comparison {
                    ComparisonResult::Improved => {
                        explanation_str = "Performance has improved.".to_owned();
                    }
                    ComparisonResult::Regressed => {
                        explanation_str = "Performance has regressed.".to_owned();
                    }
                    ComparisonResult::NonSignificant => {
                        explanation_str = "Change within noise threshold.".to_owned();
                    }
                }
            }

            let comp = Comparison {
                p_value: format!("{:.2}", comp.p_value),
                inequality: (if different_mean { "<" } else { ">" }).to_owned(),
                significance_level: format!("{:.2}", comp.significance_threshold),
                explanation: explanation_str,

                change: ConfidenceInterval {
                    point: format::change(mean_est.point_estimate, true),
                    lower: format::change(mean_est.confidence_interval.lower_bound, true),
                    upper: format::change(mean_est.confidence_interval.upper_bound, true),
                },

                thrpt_change: measurements.throughput.as_ref().map(|_| {
                    let to_thrpt_estimate = |ratio: f64| 1.0 / (1.0 + ratio) - 1.0;
                    ConfidenceInterval {
                        point: format::change(to_thrpt_estimate(mean_est.point_estimate), true),
                        lower: format::change(
                            to_thrpt_estimate(mean_est.confidence_interval.lower_bound),
                            true,
                        ),
                        upper: format::change(
                            to_thrpt_estimate(mean_est.confidence_interval.upper_bound),
                            true,
                        ),
                    }
                }),

                additional_plots: vec![
                    Plot::new("Change in mean", "change/mean.svg"),
                    Plot::new("Change in median", "change/median.svg"),
                    Plot::new("T-Test", "change/t-test.svg"),
                ],
            };
            Some(comp)
        } else {
            None
        }
    }

    fn generate_plots(
        &self,
        id: &BenchmarkId,
        context: &ReportContext,
        formatter: &dyn ValueFormatter,
        measurements: &MeasurementData<'_>,
    ) {
        let plot_ctx = PlotContext {
            id,
            context,
            size: None,
            is_thumbnail: false,
        };

        let plot_data = PlotData {
            measurements,
            formatter,
            comparison: None,
        };

        let plot_ctx_small = plot_ctx.thumbnail(true).size(THUMBNAIL_SIZE);

        self.plotter.borrow_mut().pdf(plot_ctx, plot_data);
        self.plotter.borrow_mut().pdf(plot_ctx_small, plot_data);
        if measurements.absolute_estimates.slope.is_some() {
            self.plotter.borrow_mut().regression(plot_ctx, plot_data);
            self.plotter
                .borrow_mut()
                .regression(plot_ctx_small, plot_data);
        } else {
            self.plotter
                .borrow_mut()
                .iteration_times(plot_ctx, plot_data);
            self.plotter
                .borrow_mut()
                .iteration_times(plot_ctx_small, plot_data);
        }

        self.plotter
            .borrow_mut()
            .abs_distributions(plot_ctx, plot_data);

        if let Some(ref comp) = measurements.comparison {
            try_else_return!({
                let mut change_dir = context.output_directory.clone();
                change_dir.push(id.as_directory_name());
                change_dir.push("report");
                change_dir.push("change");
                fs::mkdirp(&change_dir)
            });

            try_else_return!({
                let mut both_dir = context.output_directory.clone();
                both_dir.push(id.as_directory_name());
                both_dir.push("report");
                both_dir.push("both");
                fs::mkdirp(&both_dir)
            });

            let comp_data = plot_data.comparison(comp);

            self.plotter.borrow_mut().pdf(plot_ctx, comp_data);
            self.plotter.borrow_mut().pdf(plot_ctx_small, comp_data);
            if measurements.absolute_estimates.slope.is_some()
                && comp.base_estimates.slope.is_some()
            {
                self.plotter.borrow_mut().regression(plot_ctx, comp_data);
                self.plotter
                    .borrow_mut()
                    .regression(plot_ctx_small, comp_data);
            } else {
                self.plotter
                    .borrow_mut()
                    .iteration_times(plot_ctx, comp_data);
                self.plotter
                    .borrow_mut()
                    .iteration_times(plot_ctx_small, comp_data);
            }
            self.plotter.borrow_mut().t_test(plot_ctx, comp_data);
            self.plotter
                .borrow_mut()
                .rel_distributions(plot_ctx, comp_data);
        }

        self.plotter.borrow_mut().wait();
    }

    fn load_summary_data<'a>(
        &self,
        output_directory: &Path,
        all_ids: &[&'a BenchmarkId],
    ) -> Vec<(&'a BenchmarkId, Vec<f64>)> {
        all_ids
            .iter()
            .filter_map(|id| {
                let entry = output_directory.join(id.as_directory_name()).join("new");

                let SavedSample { iters, times, .. } =
                    try_else_return!(fs::load(&entry.join("sample.json")), || None);
                let avg_times = iters
                    .into_iter()
                    .zip(times)
                    .map(|(iters, time)| time / iters)
                    .collect::<Vec<_>>();

                Some((*id, avg_times))
            })
            .collect::<Vec<_>>()
    }

    fn generate_summary(
        &self,
        id: &BenchmarkId,
        data: &[&(&BenchmarkId, Vec<f64>)],
        report_context: &ReportContext,
        formatter: &dyn ValueFormatter,
        full_summary: bool,
        enable_comparison: bool,
    ) {
        let plot_ctx = PlotContext {
            id,
            context: report_context,
            size: None,
            is_thumbnail: false,
        };

        try_else_return!(
            {
                let mut report_dir = report_context.output_directory.clone();
                report_dir.push(id.as_directory_name());
                report_dir.push("report");
                fs::mkdirp(&report_dir)
            },
            || {}
        );

        self.plotter.borrow_mut().violin(plot_ctx, formatter, data);

        let value_types: Vec<_> = data.iter().map(|&&(id, _)| id.value_type()).collect();
        let mut line_path = None;

        if value_types.iter().all(|x| x == &value_types[0]) {
            if let Some(value_type) = value_types[0] {
                let values: Vec<_> = data.iter().map(|&&(id, _)| id.as_number()).collect();
                if values.iter().any(|x| x != &values[0]) {
                    self.plotter
                        .borrow_mut()
                        .line_comparison(plot_ctx, formatter, data, value_type);
                    line_path = Some(plot_ctx.line_comparison_path());
                }
            }
        }

        let path_prefix = if full_summary { "../.." } else { "../../.." };
        let benchmarks = data
            .iter()
            .map(|&&(id, _)| {
                IndividualBenchmark::from_id(&report_context.output_directory, path_prefix, id)
            })
            .collect();

        let comparison = if enable_comparison && report_context.comparison {
            let ids: Vec<_> = data.iter().map(|&&(id, _)| id).collect();
            let estimates = load_estimates_for_ids(&report_context.output_directory, &ids);
            if estimates.len() == ids.len() && estimates.len() > 1 {
                let change = load_change_for_ids(&report_context.output_directory, &ids);
                let rows = build_comparison_rows(&estimates, formatter, Some(&change));
                if rows.is_empty() {
                    None
                } else {
                    Some(
                        rows.into_iter()
                            .map(SummaryComparisonMetric::from)
                            .collect(),
                    )
                }
            } else {
                None
            }
        } else {
            None
        };

        let context = SummaryContext {
            group_id: id.as_title().to_owned(),

            thumbnail_width: THUMBNAIL_SIZE.unwrap().0,
            thumbnail_height: THUMBNAIL_SIZE.unwrap().1,

            violin_plot: Some(plot_ctx.violin_path().to_string_lossy().into_owned()),
            line_chart: line_path.map(|p| p.to_string_lossy().into_owned()),

            comparison,
            benchmarks,
        };

        let mut report_path = report_context.output_directory.clone();
        report_path.push(id.as_directory_name());
        report_path.push("report");
        report_path.push("index.html");
        debug_context(&report_path, &context);

        let text = self
            .templates
            .render("summary_report", &context)
            .expect("Failed to render summary report template");
        try_else_return!(fs::save_string(&text, &report_path,), || {});
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
