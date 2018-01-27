use stats::bivariate::Data;
use stats::bivariate::regression::Slope;
use report::{Report, MeasurementData};
use Criterion;

use handlebars::Handlebars;
use fs;
use format;
use estimate::{Distributions, Estimates, Statistic};
use Estimate;
use std::path::Path;
use plot;
use simplot::Size;

#[derive(Serialize)]
struct Context {
    title: String,
    confidence: String,

    slope: ConfidenceInterval,
    r2: ConfidenceInterval,
    mean: ConfidenceInterval,
    std_dev: ConfidenceInterval,
    median: ConfidenceInterval,
    mad: ConfidenceInterval,
    throughput: Option<ConfidenceInterval>,

    additional_plots: Vec<Plot>,
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

pub struct Html {
    handlebars: Handlebars,
}
impl Html {
    pub fn new() -> Html {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("report", include_str!("benchmark_report.html.handlebars")).unwrap();
        Html{ handlebars }
    }
}
impl Report for Html {
    fn benchmark_start(&self, _: &str, _: &Criterion) {}
    fn warmup(&self, _: &str, _: &Criterion, _: f64) {}
    fn analysis(&self, _: &str, _: &Criterion) {}
    fn measurement_start(&self, _: &str, _: &Criterion, _: u64, _: f64, _: u64) {}
    fn measurement_complete(&self, id: &str, criterion: &Criterion, measurements: &MeasurementData) {
        let slope_estimate = &measurements.absolute_estimates[&Statistic::Slope];

        fn time_interval(est: &Estimate) -> ConfidenceInterval {
            ConfidenceInterval {
                lower: format::time(est.confidence_interval.lower_bound),
                point: format::time(est.point_estimate),
                upper: format::time(est.confidence_interval.upper_bound)
            }
        }

        let data = Data::new(measurements.iter_counts.as_slice(), measurements.sample_times.as_slice());
        let point = Slope::fit(data);
        let slope_dist = &measurements.distributions[&Statistic::Slope];
        let (lb, ub) = slope_dist.confidence_interval(slope_estimate.confidence_interval.confidence_level);
        let (lb_, ub_) = (Slope(lb), Slope(ub));

        plot::pdf(data, measurements.avg_times, id,
            format!("{}/{}/new/pdf_small.svg", criterion.output_directory, id),
            Some(Size(550, 400)), true);
        plot::regression(data, &point, (lb_, ub_), id,
            format!("{}/{}/new/regression_small.svg", criterion.output_directory, id),
            Some(Size(550, 400)), true);

        let mut additional_plots = vec![];
        {
            let mut plot = |name: &str, file_name: &str| {
                let file_path = format!("{}/{}/new/{}", criterion.output_directory, id, file_name);
                if !Path::new(&file_path).exists() {
                    return;
                }

                let plot_struct = Plot {
                    name: name.to_owned(),
                    url: file_name.to_owned(),
                };
                additional_plots.push(plot_struct);
            };
            plot("Slope", "slope.svg");
            plot("Mean", "mean.svg");
            plot("Std. Dev.", "SD.svg");
            plot("Median", "median.svg");
            plot("MAD", "MAD.svg");
        }

        let throughput = measurements.throughput.as_ref()
            .map(|thr| {
                ConfidenceInterval{
                    lower: format::throughput(
                        thr,
                        slope_estimate.confidence_interval.upper_bound
                    ),
                    upper: format::throughput(
                        thr,
                        slope_estimate.confidence_interval.lower_bound
                    ),
                    point: format::throughput(
                        thr,
                        slope_estimate.point_estimate
                    )
                }
            });


        let context = Context {
            title: id.to_owned(),
            confidence: format!("{:.2}", slope_estimate.confidence_interval.confidence_level),

            slope: time_interval(slope_estimate),
            mean: time_interval(&measurements.absolute_estimates[&Statistic::Mean]),
            median: time_interval(&measurements.absolute_estimates[&Statistic::Median]),
            mad: time_interval(&measurements.absolute_estimates[&Statistic::MedianAbsDev]),
            std_dev: time_interval(&measurements.absolute_estimates[&Statistic::StdDev]),
            throughput: throughput,

            r2: ConfidenceInterval {
                lower: format!("{:0.7}",
                    Slope(slope_estimate.confidence_interval.lower_bound).r_squared(data)),
                upper: format!("{:0.7}",
                    Slope(slope_estimate.confidence_interval.upper_bound).r_squared(data)),
                point: format!("{:0.7}",
                    Slope(slope_estimate.point_estimate).r_squared(data)),
            },

            additional_plots: additional_plots
        };

        let text = self.handlebars.render("report", &context).unwrap();
        fs::save_string(text,
            &format!("{}/{}/new/index.html", criterion.output_directory, id)).unwrap();
    }
}
