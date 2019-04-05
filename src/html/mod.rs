use report::{make_filename_safe, BenchmarkId, MeasurementData, Report, ReportContext};
use stats::bivariate::regression::Slope;
use stats::bivariate::Data;

use criterion_plot::Size;
use estimate::Statistic;
use format;
use fs;
use plot;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::Child;
use tinytemplate::TinyTemplate;
use Estimate;

const THUMBNAIL_SIZE: Option<Size> = Some(Size(450, 300));

fn wait_on_gnuplot(children: Vec<Child>) {
    let start = ::std::time::Instant::now();
    let child_count = children.len();
    for child in children {
        match child.wait_with_output() {
            Ok(ref out) if out.status.success() => {}
            Ok(out) => error!("Error in Gnuplot: {}", String::from_utf8_lossy(&out.stderr)),
            Err(e) => error!("Got IO error while waiting for Gnuplot to complete: {}", e),
        }
    }
    let elapsed = &start.elapsed();
    info!(
        "Waiting for {} gnuplot processes took {}",
        child_count,
        ::format::time(::DurationExt::to_nanos(elapsed) as f64)
    );
}

fn debug_context<S: Serialize>(path: &str, context: &S) {
    if ::debug_enabled() {
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

    slope: ConfidenceInterval,
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
}
impl IndividualBenchmark {
    fn from_id(path_prefix: &str, id: &BenchmarkId) -> IndividualBenchmark {
        IndividualBenchmark {
            name: id.as_title().to_owned(),
            path: format!("{}/{}", path_prefix, id.as_directory_name()),
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

    benchmarks: Vec<IndividualBenchmark>,
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

fn if_exists(output_directory: &str, path: &Path) -> Option<String> {
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
    fn group(output_directory: &str, group_id: &'a str) -> ReportLink<'a> {
        let path = PathBuf::from(make_filename_safe(group_id));

        ReportLink {
            name: group_id,
            path: if_exists(output_directory, &path),
        }
    }

    fn function(output_directory: &str, group_id: &str, function_id: &'a str) -> ReportLink<'a> {
        let mut path = PathBuf::from(make_filename_safe(group_id));
        path.push(make_filename_safe(function_id));

        ReportLink {
            name: function_id,
            path: if_exists(output_directory, &path),
        }
    }

    fn value(output_directory: &str, group_id: &str, value_str: &'a str) -> ReportLink<'a> {
        let mut path = PathBuf::from(make_filename_safe(group_id));
        path.push(make_filename_safe(value_str));

        ReportLink {
            name: value_str,
            path: if_exists(output_directory, &path),
        }
    }

    fn individual(output_directory: &str, id: &'a BenchmarkId) -> ReportLink<'a> {
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
    fn new(output_directory: &str, ids: &[&'a BenchmarkId]) -> BenchmarkGroup<'a> {
        let group_id = &ids[0].group_id;
        let group_report = ReportLink::group(output_directory, group_id);

        let mut function_ids = Vec::with_capacity(ids.len());
        let mut values = Vec::with_capacity(ids.len());
        let mut individual_links = HashMap::with_capacity(ids.len());

        for id in ids.iter() {
            let function_id = id.function_id.as_ref().map(|s| s.as_str());
            let value = id.value_str.as_ref().map(|s| s.as_str());

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
                let num1 = parse_opt(&v1);
                let num2 = parse_opt(&v2);

                num1.partial_cmp(&num2).unwrap_or(Ordering::Less)
            });
            values.dedup_by_key(|os| parse_opt(&os).unwrap());
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
}
impl Html {
    pub fn new() -> Html {
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

        Html { templates }
    }
}
impl Report for Html {
    fn measurement_complete(
        &self,
        id: &BenchmarkId,
        report_context: &ReportContext,
        measurements: &MeasurementData,
    ) {
        if !report_context.plotting.is_enabled() {
            return;
        }

        try_else_return!(fs::mkdirp(&format!(
            "{}/{}/report/",
            report_context.output_directory,
            id.as_directory_name()
        )));

        let slope_estimate = &measurements.absolute_estimates[&Statistic::Slope];

        fn time_interval(est: &Estimate) -> ConfidenceInterval {
            ConfidenceInterval {
                lower: format::time(est.confidence_interval.lower_bound),
                point: format::time(est.point_estimate),
                upper: format::time(est.confidence_interval.upper_bound),
            }
        }

        let data = measurements.data;

        elapsed! {
            "Generating plots",
            self.generate_plots(id, report_context, measurements)
        }

        let throughput = measurements
            .throughput
            .as_ref()
            .map(|thr| ConfidenceInterval {
                lower: format::throughput(thr, slope_estimate.confidence_interval.upper_bound),
                upper: format::throughput(thr, slope_estimate.confidence_interval.lower_bound),
                point: format::throughput(thr, slope_estimate.point_estimate),
            });

        let context = Context {
            title: id.as_title().to_owned(),
            confidence: format!("{:.2}", slope_estimate.confidence_interval.confidence_level),

            thumbnail_width: THUMBNAIL_SIZE.unwrap().0,
            thumbnail_height: THUMBNAIL_SIZE.unwrap().1,

            slope: time_interval(slope_estimate),
            mean: time_interval(&measurements.absolute_estimates[&Statistic::Mean]),
            median: time_interval(&measurements.absolute_estimates[&Statistic::Median]),
            mad: time_interval(&measurements.absolute_estimates[&Statistic::MedianAbsDev]),
            std_dev: time_interval(&measurements.absolute_estimates[&Statistic::StdDev]),
            throughput,

            r2: ConfidenceInterval {
                lower: format!(
                    "{:0.7}",
                    Slope(slope_estimate.confidence_interval.lower_bound).r_squared(&data)
                ),
                upper: format!(
                    "{:0.7}",
                    Slope(slope_estimate.confidence_interval.upper_bound).r_squared(&data)
                ),
                point: format!(
                    "{:0.7}",
                    Slope(slope_estimate.point_estimate).r_squared(&data)
                ),
            },

            additional_plots: vec![
                Plot::new("Slope", "slope.svg"),
                Plot::new("Mean", "mean.svg"),
                Plot::new("Std. Dev.", "SD.svg"),
                Plot::new("Median", "median.svg"),
                Plot::new("MAD", "MAD.svg"),
            ],

            comparison: self.comparison(measurements),
        };

        let report_path = &format!(
            "{}/{}/report/index.html",
            report_context.output_directory,
            id.as_directory_name()
        );

        debug_context(&report_path, &context);

        let text = self
            .templates
            .render("benchmark_report", &context)
            .expect("Failed to render benchmark report template");
        try_else_return!(fs::save_string(&text, report_path,));
    }

    fn summarize(&self, context: &ReportContext, all_ids: &[BenchmarkId]) {
        if !context.plotting.is_enabled() {
            return;
        }

        let all_ids = all_ids
            .iter()
            .filter(|id| {
                fs::is_dir(&format!(
                    "{}/{}",
                    context.output_directory,
                    id.as_directory_name()
                ))
            })
            .collect::<Vec<_>>();

        let mut all_plots = vec![];
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
        if value_strs.iter().all(|os| try_parse(&*os).is_some()) {
            value_strs.sort_unstable_by(|v1, v2| {
                let num1 = try_parse(&v1);
                let num2 = try_parse(&v2);

                num1.partial_cmp(&num2).unwrap_or(Ordering::Less)
            });
            value_strs.dedup_by_key(|os| try_parse(&os).unwrap());
        } else {
            value_strs.sort_unstable();
            value_strs.dedup();
        }

        for function_id in function_ids {
            let samples_with_function: Vec<_> = data
                .iter()
                .by_ref()
                .filter(|&&(ref id, _)| id.function_id.as_ref() == Some(&function_id))
                .collect();

            if samples_with_function.len() > 1 {
                let subgroup_id =
                    BenchmarkId::new(group_id.clone(), Some(function_id.clone()), None, None);

                all_plots.extend(self.generate_summary(
                    &subgroup_id,
                    &*samples_with_function,
                    context,
                    false,
                ));
            }
        }

        for value_str in value_strs {
            let samples_with_value: Vec<_> = data
                .iter()
                .by_ref()
                .filter(|&&(ref id, _)| id.value_str.as_ref() == Some(&value_str))
                .collect();

            if samples_with_value.len() > 1 {
                let subgroup_id =
                    BenchmarkId::new(group_id.clone(), None, Some(value_str.clone()), None);

                all_plots.extend(self.generate_summary(
                    &subgroup_id,
                    &*samples_with_value,
                    context,
                    false,
                ));
            }
        }

        all_plots.extend(self.generate_summary(
            &BenchmarkId::new(group_id, None, None, None),
            &*(data.iter().by_ref().collect::<Vec<_>>()),
            context,
            true,
        ));
        wait_on_gnuplot(all_plots)
    }

    fn final_summary(&self, report_context: &ReportContext) {
        if !report_context.plotting.is_enabled() {
            return;
        }

        let output_directory = &report_context.output_directory;
        if !fs::is_dir(&output_directory) {
            return;
        }

        let mut found_ids = try_else_return!(fs::list_existing_benchmarks(&output_directory));
        found_ids.sort_unstable_by_key(|id| id.id().to_owned());

        // Group IDs by group id
        let mut id_groups: HashMap<&str, Vec<&BenchmarkId>> = HashMap::new();
        for id in found_ids.iter() {
            id_groups
                .entry(&id.group_id)
                .or_insert_with(|| vec![])
                .push(id);
        }

        let mut groups = id_groups
            .into_iter()
            .map(|(_, group)| BenchmarkGroup::new(output_directory, &group))
            .collect::<Vec<BenchmarkGroup>>();
        groups.sort_unstable_by_key(|g| g.group_report.name);

        try_else_return!(fs::mkdirp(&format!("{}/report/", output_directory)));

        let report_path = &format!("{}/report/index.html", output_directory);

        let context = IndexContext { groups };

        debug_context(&report_path, &context);

        let text = self
            .templates
            .render("index", &context)
            .expect("Failed to render index template");
        try_else_return!(fs::save_string(&text, report_path,));
    }
}
impl Html {
    fn comparison(&self, measurements: &MeasurementData) -> Option<Comparison> {
        if let Some(ref comp) = measurements.comparison {
            let different_mean = comp.p_value < comp.significance_threshold;
            let mean_est = comp.relative_estimates[&Statistic::Mean];
            let explanation_str: String;

            if !different_mean {
                explanation_str = "No change in performance detected.".to_owned();
            } else {
                let comparison = compare_to_threshold(&mean_est, comp.noise_threshold);
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
        measurements: &MeasurementData,
    ) {
        let mut gnuplots = vec![
            // Probability density plots
            plot::pdf(id, context, measurements, None),
            plot::pdf_small(id, context, measurements, THUMBNAIL_SIZE),
            // Linear regression plots
            plot::regression(id, context, measurements, None),
            plot::regression_small(id, context, measurements, THUMBNAIL_SIZE),
        ];
        gnuplots.extend(plot::abs_distributions(id, context, measurements, None));

        if let Some(ref comp) = measurements.comparison {
            try_else_return!(fs::mkdirp(&format!(
                "{}/{}/report/change/",
                context.output_directory,
                id.as_directory_name()
            )));

            try_else_return!(fs::mkdirp(&format!(
                "{}/{}/report/both",
                context.output_directory,
                id.as_directory_name()
            )));

            let base_data = Data::new(&comp.base_iter_counts, &comp.base_sample_times);
            gnuplots.append(&mut vec![
                plot::regression_comparison(id, context, measurements, comp, &base_data, None),
                plot::regression_comparison_small(
                    id,
                    context,
                    measurements,
                    comp,
                    &base_data,
                    THUMBNAIL_SIZE,
                ),
                plot::pdf_comparison(id, context, measurements, comp, None),
                plot::pdf_comparison_small(id, context, measurements, comp, THUMBNAIL_SIZE),
                plot::t_test(id, context, measurements, comp, None),
            ]);
            gnuplots.extend(plot::rel_distributions(
                id,
                context,
                measurements,
                comp,
                None,
            ));
        }

        wait_on_gnuplot(gnuplots);
    }

    fn load_summary_data<'a>(
        &self,
        output_directory: &str,
        all_ids: &[&'a BenchmarkId],
    ) -> Vec<(&'a BenchmarkId, Vec<f64>)> {
        let output_dir = Path::new(output_directory);

        all_ids
            .iter()
            .filter_map(|id| {
                let entry = output_dir.join(id.as_directory_name()).join("new");

                let (iters, times): (Vec<f64>, Vec<f64>) =
                    try_else_return!(fs::load(&entry.join("sample.json")), || None);
                let avg_times = iters
                    .into_iter()
                    .zip(times.into_iter())
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
        full_summary: bool,
    ) -> Vec<Child> {
        let mut gnuplots = vec![];

        try_else_return!(
            fs::mkdirp(&format!(
                "{}/{}/report/",
                report_context.output_directory,
                id.as_directory_name()
            )),
            || gnuplots
        );

        let violin_path = format!(
            "{}/{}/report/violin.svg",
            report_context.output_directory,
            id.as_directory_name()
        );
        gnuplots.push(plot::violin(
            id.as_title(),
            data,
            &violin_path,
            report_context.plot_config.summary_scale,
        ));

        let value_types: Vec<_> = data.iter().map(|&&(ref id, _)| id.value_type()).collect();
        let mut line_path = None;

        if value_types.iter().all(|x| x == &value_types[0]) {
            if let Some(value_type) = value_types[0] {
                let values: Vec<_> = data.iter().map(|&&(ref id, _)| id.as_number()).collect();
                if values.iter().any(|x| x != &values[0]) {
                    let path = format!(
                        "{}/{}/report/lines.svg",
                        report_context.output_directory,
                        id.as_directory_name()
                    );

                    gnuplots.push(plot::line_comparison(
                        id.as_title(),
                        data,
                        &path,
                        value_type,
                        report_context.plot_config.summary_scale,
                    ));

                    line_path = Some(path);
                }
            }
        }

        let path_prefix = if full_summary { "../.." } else { "../../.." };
        let benchmarks = data
            .iter()
            .map(|&&(ref id, _)| IndividualBenchmark::from_id(path_prefix, id))
            .collect();

        let context = SummaryContext {
            group_id: id.as_title().to_owned(),

            thumbnail_width: THUMBNAIL_SIZE.unwrap().0,
            thumbnail_height: THUMBNAIL_SIZE.unwrap().1,

            violin_plot: Some(violin_path),
            line_chart: line_path,

            benchmarks,
        };

        let report_path = &format!(
            "{}/{}/report/index.html",
            report_context.output_directory,
            id.as_directory_name()
        );

        debug_context(&report_path, &context);

        let text = self
            .templates
            .render("summary_report", &context)
            .expect("Failed to render summary report template");
        try_else_return!(fs::save_string(&text, report_path,), || gnuplots);

        gnuplots
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
