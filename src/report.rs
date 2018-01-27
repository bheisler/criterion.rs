use stats::bivariate::Data;
use stats::bivariate::regression::Slope;
use stats::univariate::outliers::tukey::LabeledSample;

use format;
use stats::univariate::Sample;
use estimate::{Distributions, Estimates, Statistic};
use Estimate;
use std::io::stdout;
use std::io::Write;
use std::cell::Cell;
use {Throughput, Criterion};

pub(crate) struct ComparisonData {
    pub p_value: f64,
    pub t_value: f64,
    pub relative_estimates: Estimates,
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

pub(crate) trait Report {
    fn benchmark_start(&self, id: &str, criterion: &Criterion);
    fn warmup(&self, id: &str, criterion: &Criterion, warmup_ns: f64);
    fn analysis(&self, id: &str, criterion: &Criterion);
    fn measurement_start(&self, id: &str, criterion: &Criterion, sample_count: u64, estimate_ns: f64, iter_count: u64);
    fn measurement_complete(&self, id: &str, criterion: &Criterion, measurements: &MeasurementData);
}

pub(crate) struct Reports {
    reports: Vec<Box<Report>>
}
impl Reports {
    pub fn new(reports: Vec<Box<Report>>) -> Reports {
        Reports{ reports }
    }
}
impl Report for Reports {
    fn benchmark_start(&self, id: &str, criterion: &Criterion) {
        for report in self.reports.iter() {
            report.benchmark_start(id, criterion);
        }
    }

    fn warmup(&self, id: &str, criterion: &Criterion, warmup_ns: f64) {
        for report in self.reports.iter() {
            report.warmup(id, criterion, warmup_ns);
        }
    }

    fn analysis(&self, id: &str, criterion: &Criterion) {
        for report in self.reports.iter() {
            report.analysis(id, criterion);
        }
    }

    fn measurement_start(&self, id: &str, criterion: &Criterion, sample_count: u64, estimate_ns: f64, iter_count: u64) {
        for report in self.reports.iter() {
            report.measurement_start(id, criterion, sample_count, estimate_ns, iter_count);
        }
    }
    fn measurement_complete(&self, id: &str, criterion: &Criterion, measurements: &MeasurementData) {
        for report in self.reports.iter() {
            report.measurement_complete(id, criterion, measurements);
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
            enable_text_overwrite: enable_text_overwrite,
            enable_text_coloring: enable_text_coloring,
            verbose: verbose,

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
    fn benchmark_start(&self, id: &str, _: &Criterion) {
        self.print_overwritable(format!("Benchmarking {}", id));
    }

    fn warmup(&self, id: &str, _: &Criterion, warmup_ns: f64) {
        self.text_overwrite();
        self.print_overwritable(format!(
            "Benchmarking {}: Warming up for {}",
            id,
            format::time(warmup_ns)
        ));
    }

    fn analysis(&self, id: &str, _: &Criterion) {
        self.text_overwrite();
        self.print_overwritable(format!("Benchmarking {}: Analyzing", id));
    }

    fn measurement_start(&self, id: &str, _: &Criterion, sample_count: u64, estimate_ns: f64, iter_count: u64) {
        self.text_overwrite();
        self.print_overwritable(format!(
            "Benchmarking {}: Collecting {} samples in estimated {} ({} iterations)",
            id,
            sample_count,
            format::time(estimate_ns),
            iter_count
        ));
    }

    fn measurement_complete(&self, id: &str, _: &Criterion, meas: &MeasurementData) {
        self.text_overwrite();

        let slope_estimate = meas.absolute_estimates[&Statistic::Slope];

        {
            let mut id = String::from(id);

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
