use stats::bivariate::Data;
use stats::bivariate::regression::Slope;
use report::{MeasurementData, Report};
use Criterion;

use handlebars::Handlebars;
use fs;
use format;
use estimate::Statistic;
use Estimate;
use plot;
use simplot::Size;
use stats::univariate::Sample;
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

pub struct Html {
    handlebars: Handlebars,
}
impl Html {
    pub fn new() -> Html {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("report", include_str!("benchmark_report.html.handlebars"))
            .unwrap();
        Html { handlebars }
    }
}
impl Report for Html {
    fn benchmark_start(&self, _: &str, _: &Criterion) {}
    fn warmup(&self, _: &str, _: &Criterion, _: f64) {}
    fn analysis(&self, _: &str, _: &Criterion) {}
    fn measurement_start(&self, _: &str, _: &Criterion, _: u64, _: f64, _: u64) {}
    fn measurement_complete(
        &self,
        id: &str,
        criterion: &Criterion,
        measurements: &MeasurementData,
    ) {
        if !criterion.plotting.is_enabled() {
            return;
        }

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
            self.generate_plots(id, criterion, measurements)
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
            title: id.to_owned(),
            confidence: format!("{:.2}", slope_estimate.confidence_interval.confidence_level),

            thumbnail_width: THUMBNAIL_SIZE.0,
            thumbnail_height: THUMBNAIL_SIZE.1,

            slope: time_interval(slope_estimate),
            mean: time_interval(&measurements.absolute_estimates[&Statistic::Mean]),
            median: time_interval(&measurements.absolute_estimates[&Statistic::Median]),
            mad: time_interval(&measurements.absolute_estimates[&Statistic::MedianAbsDev]),
            std_dev: time_interval(&measurements.absolute_estimates[&Statistic::StdDev]),
            throughput: throughput,

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

        let text = self.handlebars.render("report", &context).unwrap();
        fs::save_string(
            &text,
            &format!("{}/{}/new/index.html", criterion.output_directory, id),
        ).unwrap();
    }

    fn summarize(&self, criterion: &Criterion, group_id: &str, all_ids: &[String]) {
        if !criterion.plotting.is_enabled() {
            return;
        }

        wait_on_gnuplot(plot::summarize(
            group_id,
            all_ids,
            &criterion.output_directory,
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
                    Plot::new("Change in mean", "../change/mean.svg"),
                    Plot::new("Change in median", "../change/median.svg"),
                    Plot::new("T-Test", "../change/t-test.svg"),
                ],
            };
            Some(comp)
        } else {
            None
        }
    }

    fn generate_plots(&self, id: &str, criterion: &Criterion, measurements: &MeasurementData) {
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
            format!("{}/{}/new/pdf.svg", criterion.output_directory, id),
            None,
            false,
        ));
        gnuplots.extend(plot::abs_distributions(
            &measurements.distributions,
            &measurements.absolute_estimates,
            id,
            &criterion.output_directory,
        ));
        gnuplots.push(plot::regression(
            data,
            &point,
            (lb_, ub_),
            id,
            format!("{}/{}/new/regression.svg", criterion.output_directory, id),
            None,
            false,
        ));
        gnuplots.push(plot::pdf(
            data,
            measurements.avg_times,
            id,
            format!("{}/{}/new/pdf_small.svg", criterion.output_directory, id),
            Some(THUMBNAIL_SIZE),
            true,
        ));
        gnuplots.push(plot::regression(
            data,
            &point,
            (lb_, ub_),
            id,
            format!(
                "{}/{}/new/regression_small.svg",
                criterion.output_directory, id
            ),
            Some(THUMBNAIL_SIZE),
            true,
        ));

        if let Some(ref comp) = measurements.comparison {
            let base_data = Data::new(&comp.base_iter_counts, &comp.base_sample_times);

            log_if_err!(fs::mkdirp(&format!(
                "{}/{}/both",
                criterion.output_directory, id
            )));
            gnuplots.push(plot::both::regression(
                base_data,
                &comp.base_estimates,
                data,
                &measurements.absolute_estimates,
                id,
                format!("{}/{}/both/regression.svg", criterion.output_directory, id),
                None,
                false,
            ));
            gnuplots.push(plot::both::pdfs(
                Sample::new(&comp.base_avg_times),
                &*measurements.avg_times,
                id,
                format!("{}/{}/both/pdf.svg", criterion.output_directory, id),
                None,
                false,
            ));
            gnuplots.push(plot::t_test(
                comp.t_value,
                &comp.t_distribution,
                id,
                &criterion.output_directory,
            ));
            gnuplots.extend(plot::rel_distributions(
                &comp.relative_distributions,
                &comp.relative_estimates,
                id,
                &criterion.output_directory,
                comp.noise_threshold,
            ));
            gnuplots.push(plot::both::regression(
                base_data,
                &comp.base_estimates,
                data,
                &measurements.absolute_estimates,
                id,
                format!(
                    "{}/{}/new/relative_regression_small.svg",
                    criterion.output_directory, id
                ),
                Some(THUMBNAIL_SIZE),
                true,
            ));
            gnuplots.push(plot::both::pdfs(
                Sample::new(&comp.base_avg_times),
                &*measurements.avg_times,
                id,
                format!(
                    "{}/{}/new/relative_pdf_small.svg",
                    criterion.output_directory, id
                ),
                Some(THUMBNAIL_SIZE),
                true,
            ));
        }

        wait_on_gnuplot(gnuplots);
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
