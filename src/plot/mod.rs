use std::iter;
use std::path::PathBuf;
use std::process::Child;

use criterion_plot::prelude::*;
use stats::Distribution;
use stats::bivariate::Data;
use stats::bivariate::regression::Slope;
use stats::univariate::Sample;
use stats::univariate::outliers::tukey::LabeledSample;

use estimate::{Distributions, Estimates};
use kde;
use report::BenchmarkId;

pub mod both;
pub mod summary;

fn escape_underscores(string: &str) -> String {
    string.replace("_", "\\_")
}

fn scale_time(ns: f64) -> (f64, &'static str) {
    if ns < 10f64.powi(0) {
        (10f64.powi(3), "p")
    } else if ns < 10f64.powi(3) {
        (10f64.powi(0), "n")
    } else if ns < 10f64.powi(6) {
        (10f64.powi(-3), "u")
    } else if ns < 10f64.powi(9) {
        (10f64.powi(-6), "m")
    } else {
        (10f64.powi(-9), "")
    }
}

static DEFAULT_FONT: &'static str = "Helvetica";
static KDE_POINTS: usize = 500;
static SIZE: Size = Size(1280, 720);

const LINEWIDTH: LineWidth = LineWidth(2.);
const POINT_SIZE: PointSize = PointSize(0.75);

const DARK_BLUE: Color = Color::Rgb(31, 120, 180);
const DARK_ORANGE: Color = Color::Rgb(255, 127, 0);
const DARK_RED: Color = Color::Rgb(227, 26, 28);

fn debug_script(path: &PathBuf, figure: &Figure) {
    if ::debug_enabled() {
        let mut script_path = path.clone();
        script_path.set_extension("gnuplot");
        println!("Writing gnuplot script to {:?}", script_path);
        let result = figure.save(script_path.as_path());
        if let Err(e) = result {
            error!("Failed to write debug output: {}", e);
        }
    }
}

pub fn pdf_small(sample: &Sample<f64>, path: String, size: Option<Size>) -> Child {
    let path = PathBuf::from(path);
    let (x_scale, prefix) = scale_time(sample.max());
    let mean = sample.mean();

    let (xs, ys, mean_y) = kde::sweep_and_estimate(sample, KDE_POINTS, None, mean);
    let xs_ = Sample::new(&xs);
    let ys_ = Sample::new(&ys);

    let y_limit = ys_.max() * 1.1;
    let zeros = iter::repeat(0);

    let mut figure = Figure::new();
    figure
        .set(Font(DEFAULT_FONT))
        .set(size.unwrap_or(SIZE))
        .configure(Axis::BottomX, |a| {
            a.set(Label(format!("Average time ({}s)", prefix)))
                .set(Range::Limits(xs_.min() * x_scale, xs_.max() * x_scale))
                .set(ScaleFactor(x_scale))
        })
        .configure(Axis::LeftY, |a| {
            a.set(Label("Density (a.u.)"))
                .set(Range::Limits(0., y_limit))
        })
        .configure(Axis::RightY, |a| a.hide())
        .configure(Key, |k| k.hide())
        .plot(
            FilledCurve {
                x: &*xs,
                y1: &*ys,
                y2: zeros,
            },
            |c| {
                c.set(Axes::BottomXRightY)
                    .set(DARK_BLUE)
                    .set(Label("PDF"))
                    .set(Opacity(0.25))
            },
        )
        .plot(
            Lines {
                x: &[mean, mean],
                y: &[0., mean_y],
            },
            |c| c.set(DARK_BLUE).set(LINEWIDTH).set(Label("Mean")),
        );

    debug_script(&path, &figure);
    figure.set(Output(path)).draw().unwrap()
}

pub fn pdf(
    data: Data<f64, f64>,
    labeled_sample: LabeledSample<f64>,
    id: &BenchmarkId,
    path: String,
    size: Option<Size>,
) -> Child {
    let path = PathBuf::from(path);
    let (x_scale, prefix) = scale_time(labeled_sample.max());
    let mean = labeled_sample.mean();

    let &max_iters = data.x()
        .as_slice()
        .iter()
        .max_by_key(|&&iters| iters as u64)
        .unwrap();
    let exponent = (max_iters.log10() / 3.).floor() as i32 * 3;
    let y_scale = 10f64.powi(-exponent);

    let y_label = if exponent == 0 {
        "Iterations".to_owned()
    } else {
        format!("Iterations (x 10^{})", exponent)
    };

    let (xs, ys) = kde::sweep(&labeled_sample, KDE_POINTS, None);
    let xs_ = Sample::new(&xs);

    let (lost, lomt, himt, hist) = labeled_sample.fences();

    let vertical = &[0., max_iters];
    let zeros = iter::repeat(0);

    let mut figure = Figure::new();
    figure
        .set(Font(DEFAULT_FONT))
        .set(size.unwrap_or(SIZE))
        .configure(Axis::BottomX, |a| {
            a.set(Label(format!("Average time ({}s)", prefix)))
                .set(Range::Limits(xs_.min() * x_scale, xs_.max() * x_scale))
                .set(ScaleFactor(x_scale))
        })
        .configure(Axis::LeftY, |a| {
            a.set(Label(y_label))
                .set(Range::Limits(0., max_iters * y_scale))
                .set(ScaleFactor(y_scale))
        })
        .configure(Axis::RightY, |a| a.set(Label("Density (a.u.)")))
        .configure(Key, |k| {
            k.set(Justification::Left)
                .set(Order::SampleText)
                .set(Position::Outside(Vertical::Top, Horizontal::Right))
        })
        .plot(
            FilledCurve {
                x: &*xs,
                y1: &*ys,
                y2: zeros,
            },
            |c| {
                c.set(Axes::BottomXRightY)
                    .set(DARK_BLUE)
                    .set(Label("PDF"))
                    .set(Opacity(0.25))
            },
        )
        .plot(
            Lines {
                x: &[mean, mean],
                y: vertical,
            },
            |c| {
                c.set(DARK_BLUE)
                    .set(LINEWIDTH)
                    .set(LineType::Dash)
                    .set(Label("Mean"))
            },
        )
        .plot(
            Points {
                x: labeled_sample.iter().filter_map(|(t, label)| {
                    if label.is_outlier() {
                        None
                    } else {
                        Some(t)
                    }
                }),
                y: labeled_sample
                    .iter()
                    .zip(data.x().as_slice().iter())
                    .filter_map(
                        |((_, label), i)| {
                            if label.is_outlier() {
                                None
                            } else {
                                Some(i)
                            }
                        },
                    ),
            },
            |c| {
                c.set(DARK_BLUE)
                    .set(Label("\"Clean\" sample"))
                    .set(PointType::FilledCircle)
                    .set(POINT_SIZE)
            },
        )
        .plot(
            Points {
                x: labeled_sample.iter().filter_map(
                    |(x, label)| {
                        if label.is_mild() {
                            Some(x)
                        } else {
                            None
                        }
                    },
                ),
                y: labeled_sample
                    .iter()
                    .zip(data.x().as_slice().iter())
                    .filter_map(
                        |((_, label), i)| {
                            if label.is_mild() {
                                Some(i)
                            } else {
                                None
                            }
                        },
                    ),
            },
            |c| {
                c.set(DARK_ORANGE)
                    .set(Label("Mild outliers"))
                    .set(POINT_SIZE)
                    .set(PointType::FilledCircle)
            },
        )
        .plot(
            Points {
                x: labeled_sample.iter().filter_map(
                    |(x, label)| {
                        if label.is_severe() {
                            Some(x)
                        } else {
                            None
                        }
                    },
                ),
                y: labeled_sample
                    .iter()
                    .zip(data.x().as_slice().iter())
                    .filter_map(
                        |((_, label), i)| {
                            if label.is_severe() {
                                Some(i)
                            } else {
                                None
                            }
                        },
                    ),
            },
            |c| {
                c.set(DARK_RED)
                    .set(Label("Severe outliers"))
                    .set(POINT_SIZE)
                    .set(PointType::FilledCircle)
            },
        )
        .plot(
            Lines {
                x: &[lomt, lomt],
                y: vertical,
            },
            |c| c.set(DARK_ORANGE).set(LINEWIDTH).set(LineType::Dash),
        )
        .plot(
            Lines {
                x: &[himt, himt],
                y: vertical,
            },
            |c| c.set(DARK_ORANGE).set(LINEWIDTH).set(LineType::Dash),
        )
        .plot(
            Lines {
                x: &[lost, lost],
                y: vertical,
            },
            |c| c.set(DARK_RED).set(LINEWIDTH).set(LineType::Dash),
        )
        .plot(
            Lines {
                x: &[hist, hist],
                y: vertical,
            },
            |c| c.set(DARK_RED).set(LINEWIDTH).set(LineType::Dash),
        );
    figure.set(Title(escape_underscores(id.id())));

    debug_script(&path, &figure);
    figure.set(Output(path)).draw().unwrap()
}

pub fn regression(
    data: Data<f64, f64>,
    point: Slope<f64>,
    (lb, ub): (Slope<f64>, Slope<f64>),
    id: &BenchmarkId,
    path: String,
    size: Option<Size>,
    thumbnail_mode: bool,
) -> Child {
    let path = PathBuf::from(path);

    let (max_iters, max_elapsed) = (data.x().max(), data.y().max());

    let (y_scale, prefix) = scale_time(max_elapsed);

    let exponent = (max_iters.log10() / 3.).floor() as i32 * 3;
    let x_scale = 10f64.powi(-exponent);

    let x_label = if exponent == 0 {
        "Iterations".to_owned()
    } else {
        format!("Iterations (x 10^{})", exponent)
    };

    let lb = lb.0 * max_iters;
    let point = point.0 * max_iters;
    let ub = ub.0 * max_iters;
    let max_iters = max_iters;

    let mut figure = Figure::new();
    figure
        .set(Font(DEFAULT_FONT))
        .set(size.unwrap_or(SIZE))
        .configure(Key, |k| {
            if thumbnail_mode {
                k.hide();
            }
            k.set(Justification::Left)
                .set(Order::SampleText)
                .set(Position::Inside(Vertical::Top, Horizontal::Left))
        })
        .configure(Axis::BottomX, |a| {
            a.configure(Grid::Major, |g| g.show())
                .set(Label(x_label))
                .set(ScaleFactor(x_scale))
        })
        .configure(Axis::LeftY, |a| {
            a.configure(Grid::Major, |g| g.show())
                .set(Label(format!("Total sample time ({}s)", prefix)))
                .set(ScaleFactor(y_scale))
        })
        .plot(
            Points {
                x: data.x().as_slice(),
                y: data.y().as_slice(),
            },
            |c| {
                c.set(DARK_BLUE)
                    .set(Label("Sample"))
                    .set(PointSize(0.5))
                    .set(PointType::FilledCircle)
            },
        )
        .plot(
            Lines {
                x: &[0., max_iters],
                y: &[0., point],
            },
            |c| {
                c.set(DARK_BLUE)
                    .set(LINEWIDTH)
                    .set(Label("Linear regression"))
                    .set(LineType::Solid)
            },
        )
        .plot(
            FilledCurve {
                x: &[0., max_iters],
                y1: &[0., lb],
                y2: &[0., ub],
            },
            |c| {
                c.set(DARK_BLUE)
                    .set(Label("Confidence interval"))
                    .set(Opacity(0.25))
            },
        );
    if !thumbnail_mode {
        figure.set(Title(escape_underscores(id.id())));
    }

    debug_script(&path, &figure);
    figure.set(Output(path)).draw().unwrap()
}

pub(crate) fn abs_distributions(
    distributions: &Distributions,
    estimates: &Estimates,
    id: &BenchmarkId,
    output_directory: &str,
) -> Vec<Child> {
    distributions
        .iter()
        .map(|(&statistic, distribution)| {
            let path = PathBuf::from(format!(
                "{}/{}/report/{}.svg",
                output_directory,
                id.as_directory_name(),
                statistic
            ));
            let estimate = estimates[&statistic];

            let ci = estimate.confidence_interval;
            let (lb, ub) = (ci.lower_bound, ci.upper_bound);

            let start = lb - (ub - lb) / 9.;
            let end = ub + (ub - lb) / 9.;
            let (xs, ys) = kde::sweep(distribution, KDE_POINTS, Some((start, end)));
            let xs_ = Sample::new(&xs);

            let (x_scale, prefix) = scale_time(xs_.max());
            let y_scale = x_scale.recip();

            let p = estimate.point_estimate;

            let n_p = xs.iter().enumerate().find(|&(_, &x)| x >= p).unwrap().0;
            let y_p =
                ys[n_p - 1] + (ys[n_p] - ys[n_p - 1]) / (xs[n_p] - xs[n_p - 1]) * (p - xs[n_p - 1]);

            let zero = iter::repeat(0);

            let start = xs.iter().enumerate().find(|&(_, &x)| x >= lb).unwrap().0;
            let end = xs.iter()
                .enumerate()
                .rev()
                .find(|&(_, &x)| x <= ub)
                .unwrap()
                .0;
            let len = end - start;

            let mut figure = Figure::new();
            figure
                .set(Font(DEFAULT_FONT))
                .set(SIZE)
                .set(Title(format!(
                    "{}: {}",
                    escape_underscores(id.id()),
                    statistic
                )))
                .configure(Axis::BottomX, |a| {
                    a.set(Label(format!("Average time ({}s)", prefix)))
                        .set(Range::Limits(xs_.min() * x_scale, xs_.max() * x_scale))
                        .set(ScaleFactor(x_scale))
                })
                .configure(Axis::LeftY, |a| {
                    a.set(Label("Density (a.u.)")).set(ScaleFactor(y_scale))
                })
                .configure(Key, |k| {
                    k.set(Justification::Left)
                        .set(Order::SampleText)
                        .set(Position::Outside(Vertical::Top, Horizontal::Right))
                })
                .plot(Lines { x: &*xs, y: &*ys }, |c| {
                    c.set(DARK_BLUE)
                        .set(LINEWIDTH)
                        .set(Label("Bootstrap distribution"))
                        .set(LineType::Solid)
                })
                .plot(
                    FilledCurve {
                        x: xs.iter().skip(start).take(len),
                        y1: ys.iter().skip(start),
                        y2: zero,
                    },
                    |c| {
                        c.set(DARK_BLUE)
                            .set(Label("Confidence interval"))
                            .set(Opacity(0.25))
                    },
                )
                .plot(
                    Lines {
                        x: &[p, p],
                        y: &[0., y_p],
                    },
                    |c| {
                        c.set(DARK_BLUE)
                            .set(LINEWIDTH)
                            .set(Label("Point estimate"))
                            .set(LineType::Dash)
                    },
                );
            debug_script(&path, &figure);
            figure.set(Output(path)).draw().unwrap()
        })
        .collect::<Vec<_>>()
}

// TODO DRY: This is very similar to the `abs_distributions` method
pub(crate) fn rel_distributions(
    distributions: &Distributions,
    estimates: &Estimates,
    id: &BenchmarkId,
    output_directory: &str,
    nt: f64,
) -> Vec<Child> {
    let mut figure = Figure::new();

    figure
        .set(Font(DEFAULT_FONT))
        .set(SIZE)
        .configure(Axis::LeftY, |a| a.set(Label("Density (a.u.)")))
        .configure(Key, |k| {
            k.set(Justification::Left)
                .set(Order::SampleText)
                .set(Position::Outside(Vertical::Top, Horizontal::Right))
        });

    distributions
        .iter()
        .map(|(&statistic, distribution)| {
            let path = PathBuf::from(format!(
                "{}/{}/report/change/{}.svg",
                output_directory,
                id.as_directory_name(),
                statistic
            ));

            let estimate = estimates[&statistic];
            let ci = estimate.confidence_interval;
            let (lb, ub) = (ci.lower_bound, ci.upper_bound);

            let start = lb - (ub - lb) / 9.;
            let end = ub + (ub - lb) / 9.;
            let (xs, ys) = kde::sweep(distribution, KDE_POINTS, Some((start, end)));
            let xs_ = Sample::new(&xs);

            let p = estimate.point_estimate;
            let n_p = xs.iter().enumerate().find(|&(_, &x)| x >= p).unwrap().0;
            let y_p =
                ys[n_p - 1] + (ys[n_p] - ys[n_p - 1]) / (xs[n_p] - xs[n_p - 1]) * (p - xs[n_p - 1]);

            let one = iter::repeat(1);
            let zero = iter::repeat(0);

            let start = xs.iter().enumerate().find(|&(_, &x)| x >= lb).unwrap().0;
            let end = xs.iter()
                .enumerate()
                .rev()
                .find(|&(_, &x)| x <= ub)
                .unwrap()
                .0;
            let len = end - start;

            let x_min = xs_.min();
            let x_max = xs_.max();

            let (fc_start, fc_end) = if nt < x_min || -nt > x_max {
                let middle = (x_min + x_max) / 2.;

                (middle, middle)
            } else {
                (
                    if -nt < x_min { x_min } else { -nt },
                    if nt > x_max { x_max } else { nt },
                )
            };

            let mut figure = figure.clone();
            figure
                .set(Title(format!(
                    "{}: {}",
                    escape_underscores(id.id()),
                    statistic
                )))
                .configure(Axis::BottomX, |a| {
                    a.set(Label("Relative change (%)"))
                        .set(Range::Limits(x_min * 100., x_max * 100.))
                        .set(ScaleFactor(100.))
                })
                .plot(Lines { x: &*xs, y: &*ys }, |c| {
                    c.set(DARK_BLUE)
                        .set(LINEWIDTH)
                        .set(Label("Bootstrap distribution"))
                        .set(LineType::Solid)
                })
                .plot(
                    FilledCurve {
                        x: xs.iter().skip(start).take(len),
                        y1: ys.iter().skip(start),
                        y2: zero.clone(),
                    },
                    |c| {
                        c.set(DARK_BLUE)
                            .set(Label("Confidence interval"))
                            .set(Opacity(0.25))
                    },
                )
                .plot(
                    Lines {
                        x: &[p, p],
                        y: &[0., y_p],
                    },
                    |c| {
                        c.set(DARK_BLUE)
                            .set(LINEWIDTH)
                            .set(Label("Point estimate"))
                            .set(LineType::Dash)
                    },
                )
                .plot(
                    FilledCurve {
                        x: &[fc_start, fc_end],
                        y1: one,
                        y2: zero,
                    },
                    |c| {
                        c.set(Axes::BottomXRightY)
                            .set(DARK_RED)
                            .set(Label("Noise threshold"))
                            .set(Opacity(0.1))
                    },
                );
            debug_script(&path, &figure);
            figure.set(Output(path)).draw().unwrap()
        })
        .collect::<Vec<_>>()
}

pub fn t_test(
    t: f64,
    distribution: &Distribution<f64>,
    id: &BenchmarkId,
    output_directory: &str,
) -> Child {
    let path = PathBuf::from(format!(
        "{}/{}/report/change/t-test.svg",
        output_directory,
        id.as_directory_name()
    ));

    let (xs, ys) = kde::sweep(distribution, KDE_POINTS, None);
    let zero = iter::repeat(0);

    let mut figure = Figure::new();
    figure
        .set(Font(DEFAULT_FONT))
        .set(SIZE)
        .set(Title(format!(
            "{}: Welch t test",
            escape_underscores(id.id())
        )))
        .configure(Axis::BottomX, |a| a.set(Label("t score")))
        .configure(Axis::LeftY, |a| a.set(Label("Density")))
        .configure(Key, |k| {
            k.set(Justification::Left)
                .set(Order::SampleText)
                .set(Position::Outside(Vertical::Top, Horizontal::Right))
        })
        .plot(
            FilledCurve {
                x: &*xs,
                y1: &*ys,
                y2: zero,
            },
            |c| {
                c.set(DARK_BLUE)
                    .set(Label("t distribution"))
                    .set(Opacity(0.25))
            },
        )
        .plot(
            Lines {
                x: &[t, t],
                y: &[0, 1],
            },
            |c| {
                c.set(Axes::BottomXRightY)
                    .set(DARK_BLUE)
                    .set(LINEWIDTH)
                    .set(Label("t statistic"))
                    .set(LineType::Solid)
            },
        );
    debug_script(&path, &figure);
    figure.set(Output(path)).draw().unwrap()
}

/// Private
trait Append<T> {
    /// Private
    fn append_(self, item: T) -> Self;
}

// NB I wish this was in the standard library
impl<T> Append<T> for Vec<T> {
    fn append_(mut self, item: T) -> Vec<T> {
        self.push(item);
        self
    }
}
