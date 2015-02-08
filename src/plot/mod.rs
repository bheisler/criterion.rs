use simplot::prelude::*;
use stats::Stats;
use stats::outliers::Outliers;
use stats::outliers::Label::{HighMild, HighSevere, LowMild, LowSevere};
use stats::regression::Slope;
use std::old_io::fs::PathExtensions;
use std::iter;
use std::num::Float;
use std::str;

use estimate::Statistic;
use estimate::{Distributions, Estimate, Estimates};
use fs;
use kde;

pub mod both;

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

// TODO These should be configurable
static DEFAULT_FONT: &'static str = "Helvetica";
static KDE_POINTS: usize = 500;
static SIZE: Size = Size(1280, 720);

const LINEWIDTH: LineWidth = LineWidth(2.);
const POINT_SIZE: PointSize = PointSize(0.75);

const DARK_BLUE: Color = Color::Rgb(31, 120, 180);
const DARK_ORANGE: Color = Color::Rgb(255, 127, 0);
const DARK_RED: Color = Color::Rgb(227, 26, 28);

pub fn pdf(pairs: &[(f64, f64)], sample: &[f64], outliers: &Outliers<f64>, id: &str) {
    let path = Path::new(format!(".criterion/{}/new/pdf.svg", id));

    let (scale, prefix) = scale_time(sample.max());
    let sample = sample.iter().map(|&t| t * scale).collect::<Vec<_>>();
    let sample = &*sample;

    let max_iters = pairs.iter().map(|&(iters, _)| iters).max_by(|&iters| iters as u64).unwrap();
    let exponent = (max_iters.log10() / 3.).floor() as i32 * 3;
    let y_scale = 10f64.powi(-exponent);
    let iters = pairs.iter().map(|&(x, _)| x * y_scale).collect::<Vec<_>>();

    let y_label = if exponent == 0 {
        "Iterations".to_string()
    } else {
        format!("Iterations (x 10^{})", exponent)
    };

    let (xs, ys) = kde::sweep(sample, KDE_POINTS, None);

    let (mut lost, mut lomt, mut himt, mut hist) = outliers.fences;
    himt *= scale;
    hist *= scale;
    lomt *= scale;
    lost *= scale;

    let (nlos, nlom, nnao, nhim, nhis) = outliers.count;
    let mut mild = Vec::with_capacity(nlom + nhim);
    let mut severe = Vec::with_capacity(nlos + nhis);
    let mut normal = Vec::with_capacity(nnao);

    for (time, (&label, i)) in sample.iter().zip(outliers.labels.iter().zip(iters.iter())) {
        match label {
            HighSevere | LowSevere => severe.push((i, time)),
            LowMild | HighMild => mild.push((i, time)),
            _ => normal.push((i, time)),

        }
    }

    let vertical = &[0., max_iters * y_scale];
    let zeros = iter::repeat(0);

    let gnuplot = Figure::new().
        set(Font(DEFAULT_FONT)).
        set(Output(path)).
        set(SIZE).
        set(Title(id.to_string())).
        configure(Axis::BottomX, |a| a.
            set(Label(format!("Average time ({}s)", prefix))).
            set(Range::Limits(xs.min(), xs.max()))).
        configure(Axis::LeftY, |a| a.
            set(Label(y_label)).
            set(Range::Limits(0., max_iters * y_scale))).
        configure(Axis::RightY, |a| a.
            set(Label("Density (a.u.)"))).
        configure(Key, |k| k.
            set(Justification::Left).
            set(Order::SampleText).
            set(Position::Outside(Vertical::Top, Horizontal::Right))).
        plot(FilledCurve {
            x: &*xs,
            y1: &*ys,
            y2: zeros,
        }, |c| c.
            set(Axes::BottomXRightY).
            set(DARK_BLUE).
            set(Label("PDF")).
            set(Opacity(0.25))).
        plot(Points {
            x: normal.iter().map(|x| x.1),
            y: normal.iter().map(|x| x.0),
        }, |c| c.
            set(DARK_BLUE).
            set(Label("\"Clean\" sample")).
            set(PointType::FilledCircle).
            set(POINT_SIZE)).
        plot(Points {
            x: mild.iter().map(|x| x.1),
            y: mild.iter().map(|x| x.0),
        }, |c| c.
            set(DARK_ORANGE).
            set(Label("Mild outliers")).
            set(POINT_SIZE).
            set(PointType::FilledCircle)).
        plot(Points {
            x: severe.iter().map(|x| x.1),
            y: severe.iter().map(|x| x.0),
        }, |c| c.
            set(DARK_RED).
            set(Label("Severe outliers")).
            set(POINT_SIZE).
            set(PointType::FilledCircle)).
        plot(Lines {
            x: &[lomt, lomt],
            y: vertical,
        }, |c| c.
            set(DARK_ORANGE).
            set(LINEWIDTH).
            set(LineType::Dash)).
        plot(Lines {
            x: &[himt, himt],
            y: vertical,
        }, |c| c.
            set(DARK_ORANGE).
            set(LINEWIDTH).
            set(LineType::Dash)).
        plot(Lines {
            x: &[lost, lost],
            y: vertical,
        }, |c| c.
            set(DARK_RED).
            set(LINEWIDTH).
            set(LineType::Dash)).
        plot(Lines {
            x: &[hist, hist],
            y: vertical,
        }, |c| c.
            set(DARK_RED).
            set(LINEWIDTH).
            set(LineType::Dash)).
        draw().unwrap();

    assert_eq!(Some(""), gnuplot.wait_with_output().ok().as_ref().and_then(|po| {
        str::from_utf8(&*po.error).ok()
    }))
}

pub fn regression(
    pairs: &[(f64, f64)],
    point: &Slope<f64>,
    (lb, ub): (&Slope<f64>, &Slope<f64>),
    id: &str,
) {
    let path = Path::new(format!(".criterion/{}/new/regression.svg", id));

    let (mut max_iters, mut max_elapsed) = (0., 0.);

    for &(iters, elapsed) in pairs.iter() {
        if max_iters < iters {
            max_iters = iters;
        }

        if max_elapsed < elapsed {
            max_elapsed = elapsed;
        }
    }

    let (y_scale, prefix) = scale_time(max_elapsed);
    let elapsed = pairs.iter().map(|&(_, y)| y as f64 * y_scale).collect::<Vec<_>>();

    let exponent = (max_iters.log10() / 3.).floor() as i32 * 3;
    let x_scale = 10f64.powi(-exponent);
    let iters = pairs.iter().map(|&(x, _)| x as f64 * x_scale).collect::<Vec<_>>();

    let x_label = if exponent == 0 {
        "Iterations".to_string()
    } else {
        format!("Iterations (x 10^{})", exponent)
    };

    let lb = lb.0 * max_iters * y_scale;
    let point = point.0 * max_iters * y_scale;
    let ub = ub.0 * max_iters * y_scale;
    let max_iters = max_iters * x_scale;

    let gnuplot = Figure::new().
        set(Font(DEFAULT_FONT)).
        set(Output(path)).
        set(SIZE).
        set(Title(id.to_string())).
        configure(Key, |k| k.
            set(Justification::Left).
            set(Order::SampleText).
            set(Position::Inside(Vertical::Top, Horizontal::Left))).
        configure(Axis::BottomX, |a| a.
            configure(Grid::Major, |g| g.
                show()).
            set(Label(x_label))).
        configure(Axis::LeftY, |a| a.
            configure(Grid::Major, |g| g.
                 show()).
            set(Label(format!("Total time ({}s)", prefix)))).
        plot(Points {
            x: &*iters,
            y: &*elapsed,
        }, |c| c.
            set(DARK_BLUE).
            set(Label("Sample")).
            set(PointSize(0.5)).
            set(PointType::FilledCircle)).
        plot(Lines {
            x: &[0., max_iters],
            y: &[0., point],
        }, |c| c.
            set(DARK_BLUE).
            set(LINEWIDTH).
            set(Label("Linear regression")).
            set(LineType::Solid)).
        plot(FilledCurve {
            x: &[0., max_iters],
            y1: &[0., lb],
            y2: &[0., ub],
        }, |c| c.
            set(DARK_BLUE).
            set(Label("Confidence interval")).
            set(Opacity(0.25))).
        draw().unwrap();

    assert_eq!(Some(""), gnuplot.wait_with_output().ok().as_ref().and_then(|po| {
        str::from_utf8(&*po.error).ok()
    }))
}

pub fn abs_distributions(distributions: &Distributions, estimates: &Estimates, id: &str) {
    let gnuplots = distributions.iter().map(|(&statistic, distribution)| {
        let estimate = estimates[statistic];

        let ci = estimate.confidence_interval;
        let (lb, ub) = (ci.lower_bound, ci.upper_bound);

        let start = lb - (ub - lb) / 9.;
        let end = ub + (ub - lb) / 9.;
        let (xs, ys) = kde::sweep(distribution.as_slice(), KDE_POINTS, Some((start, end)));

        let (scale, prefix) = scale_time(xs.max());
        let rscale = scale.recip();
        let xs = xs.into_iter().map(|x| x * scale).collect::<Vec<_>>();
        let ys = ys.into_iter().map(|y| y * rscale).collect::<Vec<_>>();

        let (lb, ub) = (lb * scale, ub * scale);
        let p = estimate.point_estimate * scale;
        let n_p = xs.iter().enumerate().filter(|&(_, &x)| x >= p).next().unwrap().0;
        let y_p =
            ys[n_p - 1] + (ys[n_p] - ys[n_p - 1]) / (xs[n_p] - xs[n_p - 1]) * (p - xs[n_p - 1]);

        let zero = iter::repeat(0);

        let start = xs.iter().enumerate().filter(|&(_, &x)| x >= lb).next().unwrap().0;
        let end = xs.iter().enumerate().rev().filter(|&(_, &x)| x <= ub).next().unwrap().0;
        let len = end - start;

        Figure::new().
            set(Font(DEFAULT_FONT)).
            set(Output(Path::new(format!(".criterion/{}/new/{}.svg", id, statistic)))).
            set(SIZE).
            set(Title(format!("{}: {}", id, statistic))).
            configure(Axis::BottomX, |a| a.
                set(Label(format!("Average time ({}s)", prefix))).
                set(Range::Limits(xs.min(), xs.max()))).
            configure(Axis::LeftY, |a| a.
                set(Label("Density (a.u.)"))).
            configure(Key, |k| k.
                set(Justification::Left).
                set(Order::SampleText).
                set(Position::Outside(Vertical::Top, Horizontal::Right))).
            plot(Lines {
                x: &*xs,
                y: &*ys,
            }, |c| c.
                set(DARK_BLUE).
                set(LINEWIDTH).
                set(Label("Bootstrap distribution")).
                set(LineType::Solid)).
            plot(FilledCurve {
                x: xs.iter().skip(start).take(len),
                y1: ys.iter().skip(start),
                y2: zero,
            }, |c| c.
                set(DARK_BLUE).
                set(Label("Confidence interval")).
                set(Opacity(0.25))).
            plot(Lines {
                x: &[p, p],
                y: &[0., y_p],
            }, |c| c.
                set(DARK_BLUE).
                set(LINEWIDTH).
                set(Label("Point estimate")).
                set(LineType::Dash)).
            draw().unwrap()
    }).collect::<Vec<_>>();

    for gnuplot in gnuplots.into_iter() {
        assert_eq!(Some(""), gnuplot.wait_with_output().ok().as_ref().and_then(|po| {
            str::from_utf8(&*po.error).ok()
        }))
    }
}

// TODO DRY: This is very similar to the `abs_distributions` method
pub fn rel_distributions(
    distributions: &Distributions,
    estimates: &Estimates,
    id: &str,
    nt: f64,
) {
    let mut figure = Figure::new();

    figure.
        set(Font(DEFAULT_FONT)).
        set(SIZE).
        configure(Axis::LeftY, |a| a.
            set(Label("Density (a.u.)"))).
        configure(Key, |k| k.
            set(Justification::Left).
            set(Order::SampleText).
            set(Position::Outside(Vertical::Top, Horizontal::Right)));

    let gnuplots = distributions.iter().map(|(&statistic, distribution)| {
        let path = Path::new(format!(".criterion/{}/change/{}.svg", id, statistic));

        let estimate = estimates[statistic];
        let ci = estimate.confidence_interval;
        let (lb, ub) = (ci.lower_bound, ci.upper_bound);

        let start = lb - (ub - lb) / 9.;
        let end = ub + (ub - lb) / 9.;
        let (xs, ys) = kde::sweep(distribution.as_slice(), KDE_POINTS, Some((start, end)));
        let xs = xs.into_iter().map(|x| x * 100.).collect::<Vec<_>>();
        let xs = &*xs;
        let ys = &*ys;

        let nt = nt * 100.;
        let (lb, ub) = (lb * 100., ub * 100.);
        let p = estimate.point_estimate * 100.;
        let n_p = xs.iter().enumerate().filter(|&(_, &x)| x >= p).next().unwrap().0;
        let y_p =
            ys[n_p - 1] + (ys[n_p] - ys[n_p - 1]) / (xs[n_p] - xs[n_p - 1]) * (p - xs[n_p - 1]);

        let one = iter::repeat(1);
        let zero = iter::repeat(0);

        let start = xs.iter().enumerate().filter(|&(_, &x)| x >= lb).next().unwrap().0;
        let end = xs.iter().enumerate().rev().filter(|&(_, &x)| x <= ub).next().unwrap().0;
        let len = end - start;

        let x_min = xs.min();
        let x_max = xs.max();

        let (fc_start, fc_end) = if nt < x_min || -nt > x_max {
            let middle = (x_min + x_max) / 2.;

            (middle, middle)
        } else {
            (if -nt < x_min { x_min } else { -nt }, if nt > x_max { x_max } else { nt })
        };

        figure.clone().
            set(Output(path)).
            set(Title(format!("{}: {}", id, statistic))).
            configure(Axis::BottomX, |a| a.
                set(Label("Relative change (%)")).
                set(Range::Limits(x_min, x_max))).
            plot(Lines {
                x: xs,
                y: ys,
            }, |c| c.
                set(DARK_BLUE).
                set(LINEWIDTH).
                set(Label("Bootstrap distribution")).
                set(LineType::Solid)).
            plot(FilledCurve {
                x: xs.iter().skip(start).take(len),
                y1: ys.iter().skip(start),
                y2: zero.clone(),
            }, |c| c.
                set(DARK_BLUE).
                set(Label("Confidence interval")).
                set(Opacity(0.25))).
            plot(Lines {
                x: &[p, p],
                y: &[0., y_p],
            }, |c| c.
                set(DARK_BLUE).
                set(LINEWIDTH).
                set(Label("Point estimate")).
                set(LineType::Dash)).
            plot(FilledCurve {
                x: &[fc_start, fc_end],
                y1: one,
                y2: zero,
            }, |c| c.
                set(Axes::BottomXRightY).
                set(DARK_RED).
                set(Label("Noise threshold")).
                set(Opacity(0.1))).
            draw().unwrap()
    }).collect::<Vec<_>>();

    // FIXME This sometimes fails!
    for gnuplot in gnuplots.into_iter() {
        assert_eq!(Some(""), gnuplot.wait_with_output().ok().as_ref().and_then(|po| {
            str::from_utf8(&*po.error).ok()
        }))
    }
}

pub fn t_test(t: f64, distribution: &[f64], id: &str) {
    let path = Path::new(format!(".criterion/{}/change/t-test.svg", id));

    let (xs, ys) = kde::sweep(distribution, KDE_POINTS, None);
    let ys = &*ys;
    let zero = iter::repeat(0);

    let gnuplot = Figure::new().
        set(Font(DEFAULT_FONT)).
        set(Output(path)).
        set(SIZE).
        set(Title(format!("{}: Welch t test", id))).
        configure(Axis::BottomX, |a| a.
            set(Label("t score"))).
        configure(Axis::LeftY, |a| a.
            set(Label("Density"))).
        configure(Key, |k| k.
            set(Justification::Left).
            set(Order::SampleText).
            set(Position::Outside(Vertical::Top, Horizontal::Right))).
        plot(FilledCurve {
            x: &*xs,
            y1: &*ys,
            y2: zero,
        }, |c| c.
            set(DARK_BLUE).
            set(Label("t distribution")).
            set(Opacity(0.25))).
        plot(Lines {
            x: &[t, t],
            y: &[0, 1],
        }, |c| c.
            set(Axes::BottomXRightY).
            set(DARK_BLUE).
            set(LINEWIDTH).
            set(Label("t statistic")).
            set(LineType::Solid)).
        draw().unwrap();

    assert_eq!(Some(""), gnuplot.wait_with_output().ok().as_ref().and_then(|po| {
        str::from_utf8(&*po.error).ok()
    }))
}

fn log_ceil(x: f64) -> f64 {
    let t = (10f64).powi(x.log10().floor() as i32);
    (x / t).ceil() * t
}

fn log_floor(x: f64) -> f64 {
    let t = (10f64).powi(x.log10().floor() as i32);
    (x / t).floor() * t
}

trait Append<T> {
    fn append_(self, item: T) -> Self;
}

// NB I wish this was in the standard library
impl<T> Append<T> for Vec<T> {
    fn append_(mut self, item: T) -> Vec<T> {
        self.push(item);
        self
    }
}

pub fn summarize(id: &str) {
    fn diff(s: &[f64]) -> Vec<f64> {
        s.windows(2).map(|w| w[1] - w[0]).collect()
    }

    let dir = Path::new(".criterion").join(id);
    let contents = fs::ls(&dir);

    // XXX Plot both summaries?
    for &sample in ["new", "base"].iter() {
        let mut benches = contents.iter().filter_map(|entry| {
            if entry.is_dir() && entry.filename_str() != Some("summary") {
                let label = entry.filename_str().unwrap();
                let root = entry.join(sample);

                if let Some(estimates) = Estimate::load(&root.join("estimates.json")) {
                    let sample: Vec<(u64, u64)> = fs::load(&root.join("sample.json"));
                    let sample = sample.into_iter().map(|(iters, time)| {
                        time as f64 / iters as f64
                    }).collect::<Vec<_>>();

                    Some((label, label.parse::<usize>(), estimates, sample))
                } else {
                    None
                }
            } else {
                None
            }
        }).collect::<Vec<_>>();

        if benches.is_empty() {
            continue;
        }

        fs::mkdirp(&dir.join(format!("summary/{}", sample)));

        let gnuplots = if benches.iter().all(|&(_, ref input, _, _)| input.is_ok()) {
            // TODO trendline
            let mut benches = benches.into_iter().map(|(label, input, estimates, sample)| {
                (label, input.unwrap(), estimates, sample)
            }).collect::<Vec<_>>();

            benches.sort_by(|&(_, a, _, _), &(_, b, _, _)| {
                a.cmp(&b)
            });

            [Statistic::Mean, Statistic::Median, Statistic::Slope].iter().map(|&statistic| {
                let points = benches.iter().map(|&(_, _, ref estimates, _)| {
                    estimates[statistic].point_estimate
                }).collect::<Vec<_>>();
                let lbs = benches.iter().map(|&(_, _, ref estimates, _)| {
                    estimates[statistic].confidence_interval.lower_bound
                }).collect::<Vec<_>>();
                let ubs = benches.iter().map(|&(_, _, ref estimates, _)| {
                    estimates[statistic].confidence_interval.upper_bound
                }).collect::<Vec<_>>();

                // XXX scale inputs?
                let inputs = benches.iter().map(|&(_, input, _, _)| input).collect::<Vec<_>>();

                let (scale, prefix) = scale_time(ubs.max());
                let points = points.iter().map(|&x| x * scale).collect::<Vec<_>>();
                let lbs = lbs.iter().map(|&x| x * scale).collect::<Vec<_>>();
                let ubs = ubs.iter().map(|&x| x * scale).collect::<Vec<_>>();

                // XXX Logscale triggering may need tweaking
                let xscale = if inputs.len() < 3 {
                    Scale::Linear
                } else {
                    let inputs = inputs.iter().map(|&x| x as f64).collect::<Vec<_>>();
                    let linear = diff(&*inputs).std_dev(None);
                    let log = {
                        let v = inputs.iter().map(|x| x.ln()).collect::<Vec<_>>();
                        diff(&*v).std_dev(None)
                    };

                    if linear < log {
                        Scale::Linear
                    } else {
                        Scale::Logarithmic
                    }
                };

                let yscale = if points.len() < 3 {
                    Scale::Linear
                } else {
                    let linear = diff(&*points).std_dev(None);
                    let log = {
                        let v = points.iter().map(|x| x.ln()).collect::<Vec<_>>();
                        diff(&*v).std_dev(None)
                    };

                    if linear < log {
                        Scale::Linear
                    } else {
                        Scale::Logarithmic
                    }
                };

                // TODO Review axis scaling
                Figure::new().
                    set(Font(DEFAULT_FONT)).
                    set(Output(dir.join(format!("summary/{}/{}s.svg", sample, statistic)))).
                    set(SIZE).
                    set(Title(format!("{}", id))).
                    configure(Axis::BottomX, |a| a.
                        configure(Grid::Major, |g| g.
                            show()).
                        configure(Grid::Minor, |g| match xscale {
                            Scale::Linear => g.hide(),
                            Scale::Logarithmic => g.show(),
                        }).
                        set(Label("Input")).
                        set(xscale)).
                    configure(Axis::BottomX, |a| match xscale {
                        Scale::Linear => a,
                        Scale::Logarithmic => {
                            let start = inputs[0] as f64;
                            let end = *inputs.last().unwrap() as f64;

                            a.set(Range::Limits(log_floor(start), log_ceil(end)))
                        },
                    }).
                    configure(Axis::LeftY, |a| a.
                        configure(Grid::Major, |g| g.
                            show()).
                        configure(Grid::Minor, |g| match xscale {
                            Scale::Linear => g.hide(),
                            Scale::Logarithmic => g.show(),
                        }).
                        set(Label(format!("Average time ({}s)", prefix))).
                        set(yscale)).
                    configure(Axis::LeftY, |a| match yscale {
                        Scale::Linear => a,
                        Scale::Logarithmic => {
                            let start = lbs.min();
                            let end = ubs.max();

                            a.set(Range::Limits(log_floor(start), log_ceil(end)))
                        },
                    }).
                    configure(Key, |k| k.
                        set(Justification::Left).
                        set(Order::SampleText).
                        set(Position::Inside(Vertical::Top, Horizontal::Left))).
                        plot(YErrorBars {
                            x: &*inputs,
                            y: &*points,
                            y_low: &*lbs,
                            y_high: &*ubs,
                        }, |e| e.
                        set(LINEWIDTH).
                        set(Label(format!("{}", statistic))).
                        set(POINT_SIZE).
                        set(PointType::FilledCircle)).
                    draw().unwrap()
            }).collect::<Vec<_>>()
        } else {
            // NB median go last because we reuse the ordered set in the next step (summary)
            [Statistic::Mean, Statistic::Slope, Statistic::Median].iter().map(|&statistic| {
                benches.sort_by(|&(_, _, ref a, _), &(_, _, ref b, _)| {
                    let a = a[statistic].point_estimate;
                    let b = b[statistic].point_estimate;
                    b.partial_cmp(&a).unwrap()
                });

                let points = benches.iter().map(|&(_, _, ref estimates, _)| {
                    estimates[statistic].point_estimate
                }).collect::<Vec<_>>();
                let lbs = benches.iter().map(|&(_, _, ref estimates, _)| {
                    estimates[statistic].confidence_interval.lower_bound
                }).collect::<Vec<_>>();
                let ubs = benches.iter().map(|&(_, _, ref estimates, _)| {
                    estimates[statistic].confidence_interval.upper_bound
                }).collect::<Vec<_>>();

                let (scale, prefix) = scale_time(ubs.max());
                let points = points.iter().map(|&x| x * scale).collect::<Vec<_>>();
                let lbs = lbs.iter().map(|&x| x * scale).collect::<Vec<_>>();
                let ubs = ubs.iter().map(|&x| x * scale).collect::<Vec<_>>();

                let xscale = if points.len() < 3 {
                    Scale::Linear
                } else {
                    let linear = diff(&*points).std_dev(None);
                    let log = {
                        let v = points.iter().map(|x| x.ln()).collect::<Vec<_>>();
                        diff(&*v).std_dev(None)
                    };

                    if linear < log {
                        Scale::Linear
                    } else {
                        Scale::Logarithmic
                    }
                };

                let min = *points.last().unwrap();
                let rel = points.iter().map(|&x| format!("{:.02}", x / min)).collect::<Vec<_>>();

                let tics = iter::count(0.5, 1f64);
                // TODO Review axis scaling
                Figure::new().
                    set(Font(DEFAULT_FONT)).
                    set(Output(dir.join(format!("summary/{}/{}s.svg", sample, statistic)))).
                    set(SIZE).
                    set(Title(format!("{}: Estimates of the {}s", id, statistic))).
                    configure(Axis::BottomX, |a| a.
                        configure(Grid::Major, |g| g.
                            show()).
                        configure(Grid::Minor, |g| match xscale {
                            Scale::Linear => g.hide(),
                            Scale::Logarithmic => g.show(),
                        }).
                        set(Label(format!("Average time ({}s)", prefix))).
                        set(xscale)).
                    configure(Axis::BottomX, |a| match xscale {
                        Scale::Linear => a,
                        Scale::Logarithmic => {
                            let start = lbs.min();
                            let end = ubs.max();

                            a.set(Range::Limits(log_floor(start), log_ceil(end)))
                        },
                    }).
                    configure(Axis::LeftY, |a| a.
                        set(Label("Input")).
                        set(Range::Limits(0., benches.len() as f64)).
                        set(TicLabels {
                            positions: tics.clone(),
                            labels: benches.iter().map(|&(label, _, _, _)| label),
                        })).
                    configure(Axis::RightY, |a| a.
                        set(Label("Relative time")).
                        set(Range::Limits(0., benches.len() as f64)).
                        set(TicLabels {
                            positions: tics,
                            labels: rel.iter().map(|x| x.as_slice()),
                        })).
                    plot(XErrorBars {
                        x: &*points,
                        y: iter::count(0.5, 1f64),
                        x_low: &*lbs,
                        x_high: &*ubs,
                    }, |eb| eb.
                        set(LINEWIDTH).
                        set(Label("Confidence Interval")).
                        set(POINT_SIZE).
                        set(PointType::FilledCircle)).
                    draw().unwrap()
            }).collect::<Vec<_>>().append_({
                let kdes = benches.iter().map(|&(_, _, _, ref sample)| {
                    let (x, mut y) = kde::sweep(&**sample, KDE_POINTS, None);
                    let y_max = y.max();
                    for y in y.iter_mut() {
                        *y /= y_max;
                    }

                    (x, y)
                }).collect::<Vec<_>>();
                let medians = benches.iter().map(|&(_, _, _, ref sample)| {
                    sample.percentiles().median()
                }).collect::<Vec<_>>();
                let mut xs = kdes.iter().flat_map(|&(ref x, _)| x.iter());
                let (mut min, mut max) = {
                    let first = *xs.next().unwrap();
                    (first, first)
                };
                for &e in xs {
                    if e < min {
                        min = e;
                    } else if e > max {
                        max = e;
                    }
                }
                let (scale, prefix) = scale_time(max);
                min *= scale;
                max *= scale;

                let xscale = if medians.len() < 3 {
                    Scale::Linear
                } else {
                    let linear = diff(&*medians).std_dev(None);
                    let log = {
                        let v = medians.iter().map(|x| x.ln()).collect::<Vec<_>>();
                        diff(&*v).std_dev(None)
                    };

                    if linear < log {
                        Scale::Linear
                    } else {
                        Scale::Logarithmic
                    }
                };

                let tics = iter::count(0.5, 1f64);
                let mut f = Figure::new();
                f.
                    set(Font(DEFAULT_FONT)).
                    set(Output(dir.join(format!("summary/{}/violin_plot.svg", sample)))).
                    set(SIZE).
                    set(Title(format!("{}: Violin plot", id))).
                    configure(Axis::BottomX, |a| a.
                        configure(Grid::Major, |g| g.
                            show()).
                        configure(Grid::Minor, |g| match xscale {
                            Scale::Linear => g.hide(),
                            Scale::Logarithmic => g.show(),
                        }).
                        set(Label(format!("Average time ({}s)", prefix))).
                        set(xscale)).
                    configure(Axis::BottomX, |a| match xscale {
                        Scale::Linear => a,
                        Scale::Logarithmic => {
                            a.set(Range::Limits(log_floor(min), log_ceil(max)))
                        },
                    }).
                    configure(Axis::LeftY, |a| a.
                        set(Label("Input")).
                        set(Range::Limits(0., benches.len() as f64)).
                        set(TicLabels {
                            positions: tics.clone(),
                            labels: benches.iter().map(|&(label, _, _, _)| label),
                        })).
                        plot(Points {
                            x: medians.iter().map(|&median| median * scale),
                            y: tics,
                        }, |c| c.
                        set(Color::Black).
                        set(Label("Median")).
                        set(PointType::Plus).
                        set(PointSize(2. * POINT_SIZE.0)));

                let mut is_first = true;
                for (i, &(ref x, ref y)) in kdes.iter().enumerate() {
                    let i = i as f64 + 0.5;
                    let x = x.iter().map(|&x| x * scale);
                    let y1 = y.iter().map(|&y| i + y * 0.5);
                    let y2 = y.iter().map(|&y| i - y * 0.5);

                    f.plot(FilledCurve {
                        x: x,
                        y1: y1,
                        y2: y2,
                    }, |c| if is_first {
                        is_first = false;

                        c.set(DARK_BLUE).set(Label("PDF")).set(Opacity(0.25))
                    } else {
                        c.set(DARK_BLUE).set(Opacity(0.25))
                    });
                }

                f.draw().unwrap()
            })
        };

        for gnuplot in gnuplots.into_iter() {
            assert_eq!(Some(""), gnuplot.wait_with_output().ok().as_ref().and_then(|po| {
                str::from_utf8(&*po.error).ok()
            }))
        }
    }
}
