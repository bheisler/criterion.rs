use report::{BenchmarkId, MeasurementData, Report, ReportContext};
use stats::bivariate::Data;
use stats::bivariate::regression::Slope;

use Estimate;
use criterion_plot::Size;
use estimate::Statistic;
use format;
use fs;
use handlebars::Handlebars;
use plot;
use stats::univariate::Sample;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Child;

const THUMBNAIL_SIZE: Size = Size(450, 300);

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
            name: id.id().to_owned(),
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
    additional_plots: Vec<Plot>,
}

#[derive(Serialize)]
struct IndexBenchmark {
    name: String,
    path: Option<String>,
    sub_benchmarks: Vec<IndexBenchmark>,
}
impl IndexBenchmark {
    fn add(&mut self, names: &[&str], idb: IndexBenchmark) {
        if names.is_empty() {
            if !self.sub_benchmarks.iter().any(|sub| sub.name == idb.name) {
                self.sub_benchmarks.push(idb);
            }
            return;
        }

        let name = names[0];

        for sub in &mut self.sub_benchmarks {
            if sub.name == name {
                sub.add(&names[1..], idb);
                break;
            }
        }
    }
}
#[derive(Serialize)]
struct IndexContext {
    benchmarks: Vec<IndexBenchmark>,
}

pub struct Html {
    handlebars: Handlebars,
}
impl Html {
    pub fn new() -> Html {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("report", include_str!("benchmark_report.html.handlebars"))
            .expect("Unable to parse benchmark report template.");
        handlebars
            .register_template_string(
                "summary_report",
                include_str!("summary_report.html.handlebars"),
            )
            .expect("Unable to parse summary report template.");
        handlebars
            .register_template_string("index", include_str!("index.html.handlebars"))
            .expect("Unable to parse index report template.");
        Html { handlebars }
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

        let data = Data::new(
            measurements.iter_counts.as_slice(),
            measurements.sample_times.as_slice(),
        );

        elapsed!{
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
            title: id.id().to_owned(),
            confidence: format!("{:.2}", slope_estimate.confidence_interval.confidence_level),

            thumbnail_width: THUMBNAIL_SIZE.0,
            thumbnail_height: THUMBNAIL_SIZE.1,

            slope: time_interval(slope_estimate),
            mean: time_interval(&measurements.absolute_estimates[&Statistic::Mean]),
            median: time_interval(&measurements.absolute_estimates[&Statistic::Median]),
            mad: time_interval(&measurements.absolute_estimates[&Statistic::MedianAbsDev]),
            std_dev: time_interval(&measurements.absolute_estimates[&Statistic::StdDev]),
            throughput,

            r2: ConfidenceInterval {
                lower: format!(
                    "{:0.7}",
                    Slope(slope_estimate.confidence_interval.lower_bound).r_squared(data)
                ),
                upper: format!(
                    "{:0.7}",
                    Slope(slope_estimate.confidence_interval.upper_bound).r_squared(data)
                ),
                point: format!(
                    "{:0.7}",
                    Slope(slope_estimate.point_estimate).r_squared(data)
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

        let text = self.handlebars
            .render("report", &context)
            .expect("Failed to render benchmark report template");
        try_else_return!(fs::save_string(
            &text,
            &format!(
                "{}/{}/report/index.html",
                report_context.output_directory,
                id.as_directory_name()
            ),
        ));
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
            .cloned()
            .collect::<Vec<_>>();

        let mut all_plots = vec![];
        let group_id = all_ids[0].group_id.clone();

        let data: Vec<(BenchmarkId, Vec<f64>)> =
            self.load_summary_data(&context.output_directory, &all_ids);

        let mut function_ids = BTreeSet::new();
        for id in all_ids {
            if let Some(function_id) = id.function_id {
                function_ids.insert(function_id);
            }
        }

        for function_id in function_ids {
            let samples_with_function: Vec<_> = data.iter()
                .by_ref()
                .filter(|&&(ref id, _)| id.function_id.as_ref() == Some(&function_id))
                .collect();

            if samples_with_function.len() > 1 {
                let subgroup_id = BenchmarkId::new(group_id.clone(), Some(function_id), None, None);

                all_plots.extend(self.generate_summary(
                    &subgroup_id,
                    &*samples_with_function,
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
        //TODO: Once criterion.rs moves to a proc-macro test harness, we should ensure that we have
        //all of the test IDs together in one place so we don't have to scan the file system to
        //generate this index.

        let output_directory = &report_context.output_directory;
        if !fs::is_dir(&output_directory) {
            return;
        }

        fn to_components(id: &BenchmarkId) -> Vec<&str> {
            let mut components: Vec<&str> = vec![&id.group_id];
            if let Some(ref name) = id.function_id {
                components.push(&**name);
            }
            if let Some(ref name) = id.value_str {
                components.push(&**name);
            }
            components
        }

        let mut found_ids = try_else_return!(fs::list_existing_benchmarks(&output_directory));
        found_ids.sort_unstable_by_key(|id| id.id().to_owned());

        let mut root_id = IndexBenchmark {
            name: "".to_owned(),
            path: None,
            sub_benchmarks: vec![],
        };

        for id in found_ids {
            let mut name_components = vec![];
            let mut path = PathBuf::new();
            for (name_component, path_component) in to_components(&id)
                .into_iter()
                .zip(id.as_directory_name().split('/'))
            {
                path.push(path_component);

                let report_path = path.join("report/index.html");
                let report_path = if PathBuf::from(output_directory).join(&report_path).is_file() {
                    Some(report_path.to_string_lossy().to_string())
                } else {
                    None
                };

                let sub_benchmark = IndexBenchmark {
                    name: name_component.to_owned(),
                    path: report_path,
                    sub_benchmarks: vec![],
                };

                root_id.add(&name_components, sub_benchmark);

                name_components.push(name_component);
            }
        }

        let benchmarks = root_id.sub_benchmarks;

        try_else_return!(fs::mkdirp(&format!("{}/report/", output_directory)));

        let context = IndexContext { benchmarks };

        let text = self.handlebars
            .render("index", &context)
            .expect("Failed to render index template");
        try_else_return!(fs::save_string(
            &text,
            &format!("{}/report/index.html", output_directory),
        ));
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
        let data = Data::new(
            measurements.iter_counts.as_slice(),
            measurements.sample_times.as_slice(),
        );
        let slope_estimate = &measurements.absolute_estimates[&Statistic::Slope];
        let point = Slope::fit(data);
        let slope_dist = &measurements.distributions[&Statistic::Slope];
        let (lb, ub) =
            slope_dist.confidence_interval(slope_estimate.confidence_interval.confidence_level);
        let (lb_, ub_) = (Slope(lb), Slope(ub));

        let mut gnuplots = vec![];

        gnuplots.push(plot::pdf(
            data,
            measurements.avg_times,
            id,
            format!(
                "{}/{}/report/pdf.svg",
                context.output_directory,
                id.as_directory_name()
            ),
            None,
        ));
        gnuplots.extend(plot::abs_distributions(
            &measurements.distributions,
            &measurements.absolute_estimates,
            id,
            &context.output_directory,
        ));
        gnuplots.push(plot::regression(
            data,
            point,
            (lb_, ub_),
            id,
            format!(
                "{}/{}/report/regression.svg",
                context.output_directory,
                id.as_directory_name()
            ),
            None,
            false,
        ));
        gnuplots.push(plot::pdf_small(
            &*measurements.avg_times,
            format!(
                "{}/{}/report/pdf_small.svg",
                context.output_directory,
                id.as_directory_name()
            ),
            Some(THUMBNAIL_SIZE),
        ));
        gnuplots.push(plot::regression(
            data,
            point,
            (lb_, ub_),
            id,
            format!(
                "{}/{}/report/regression_small.svg",
                context.output_directory,
                id.as_directory_name()
            ),
            Some(THUMBNAIL_SIZE),
            true,
        ));

        if let Some(ref comp) = measurements.comparison {
            try_else_return!(fs::mkdirp(&format!(
                "{}/{}/report/change/",
                context.output_directory,
                id.as_directory_name()
            )));

            let base_data = Data::new(&comp.base_iter_counts, &comp.base_sample_times);

            try_else_return!(fs::mkdirp(&format!(
                "{}/{}/report/both",
                context.output_directory,
                id.as_directory_name()
            )));
            gnuplots.push(plot::both::regression(
                base_data,
                &comp.base_estimates,
                data,
                &measurements.absolute_estimates,
                id,
                format!(
                    "{}/{}/report/both/regression.svg",
                    context.output_directory,
                    id.as_directory_name()
                ),
                None,
                false,
            ));
            gnuplots.push(plot::both::pdfs(
                Sample::new(&comp.base_avg_times),
                &*measurements.avg_times,
                id,
                format!(
                    "{}/{}/report/both/pdf.svg",
                    context.output_directory,
                    id.as_directory_name()
                ),
                None,
                false,
            ));
            gnuplots.push(plot::t_test(
                comp.t_value,
                &comp.t_distribution,
                id,
                &context.output_directory,
            ));
            gnuplots.extend(plot::rel_distributions(
                &comp.relative_distributions,
                &comp.relative_estimates,
                id,
                &context.output_directory,
                comp.noise_threshold,
            ));
            gnuplots.push(plot::both::regression(
                base_data,
                &comp.base_estimates,
                data,
                &measurements.absolute_estimates,
                id,
                format!(
                    "{}/{}/report/relative_regression_small.svg",
                    context.output_directory,
                    id.as_directory_name()
                ),
                Some(THUMBNAIL_SIZE),
                true,
            ));
            gnuplots.push(plot::both::pdfs(
                Sample::new(&comp.base_avg_times),
                &*measurements.avg_times,
                id,
                format!(
                    "{}/{}/report/relative_pdf_small.svg",
                    context.output_directory,
                    id.as_directory_name()
                ),
                Some(THUMBNAIL_SIZE),
                true,
            ));
        }

        wait_on_gnuplot(gnuplots);
    }

    fn load_summary_data(
        &self,
        output_directory: &str,
        all_ids: &[BenchmarkId],
    ) -> Vec<(BenchmarkId, Vec<f64>)> {
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

                Some((id.clone(), avg_times))
            })
            .collect::<Vec<_>>()
    }

    fn generate_summary(
        &self,
        id: &BenchmarkId,
        data: &[&(BenchmarkId, Vec<f64>)],
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
        gnuplots.push(plot::summary::violin(
            id.id(),
            data,
            &violin_path,
            report_context.plot_config.summary_scale,
        ));

        let value_types: Vec<_> = data.iter().map(|&&(ref id, _)| id.value_type()).collect();
        let mut line_path = None;

        if value_types.iter().all(|x| x == &value_types[0]) {
            if let Some(value_type) = value_types[0] {
                let path = format!(
                    "{}/{}/report/lines.svg",
                    report_context.output_directory,
                    id.as_directory_name()
                );

                gnuplots.push(plot::summary::line_comparison(
                    id.id(),
                    data,
                    &path,
                    value_type,
                    report_context.plot_config.summary_scale,
                ));

                line_path = Some(path);
            }
        }

        let path_prefix = if full_summary { "../.." } else { "../../.." };
        let benchmarks = data.iter()
            .map(|&&(ref id, _)| IndividualBenchmark::from_id(path_prefix, id))
            .collect();

        let context = SummaryContext {
            group_id: id.id().to_owned(),

            thumbnail_width: THUMBNAIL_SIZE.0,
            thumbnail_height: THUMBNAIL_SIZE.1,

            violin_plot: Some(violin_path),
            line_chart: line_path,

            benchmarks,
        };

        let text = self.handlebars
            .render("summary_report", &context)
            .expect("Failed to render summary report template");
        try_else_return!(
            fs::save_string(
                &text,
                &format!(
                    "{}/{}/report/index.html",
                    report_context.output_directory,
                    id.as_directory_name()
                ),
            ),
            || gnuplots
        );

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
