use std::iter;
use std::process::Child;

use criterion_plot::prelude::*;
use stats::univariate::Sample;
use stats::Distribution;

use super::*;
use estimate::Statistic;
use kde;
use report::{BenchmarkId, ComparisonData, MeasurementData, ReportContext};
use Estimate;

fn abs_distribution(
    id: &BenchmarkId,
    context: &ReportContext,
    statistic: Statistic,
    distribution: &Distribution<f64>,
    estimate: &Estimate,
    size: Option<Size>,
) -> Child {
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
    let y_p = ys[n_p - 1] + (ys[n_p] - ys[n_p - 1]) / (xs[n_p] - xs[n_p - 1]) * (p - xs[n_p - 1]);

    let zero = iter::repeat(0);

    let start = xs.iter().enumerate().find(|&(_, &x)| x >= lb).unwrap().0;
    let end = xs
        .iter()
        .enumerate()
        .rev()
        .find(|&(_, &x)| x <= ub)
        .unwrap()
        .0;
    let len = end - start;

    let mut figure = Figure::new();
    figure
        .set(Font(DEFAULT_FONT))
        .set(size.unwrap_or(SIZE))
        .set(Title(format!(
            "{}: {}",
            escape_underscores(id.as_title()),
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

    let path = context.report_path(id, &format!("{}.svg", statistic));
    debug_script(&path, &figure);
    figure.set(Output(path)).draw().unwrap()
}

pub(crate) fn abs_distributions(
    id: &BenchmarkId,
    context: &ReportContext,
    measurements: &MeasurementData,
    size: Option<Size>,
) -> Vec<Child> {
    measurements
        .distributions
        .iter()
        .map(|(&statistic, distribution)| {
            abs_distribution(
                id,
                context,
                statistic,
                distribution,
                &measurements.absolute_estimates[&statistic],
                size,
            )
        })
        .collect::<Vec<_>>()
}

fn rel_distribution(
    id: &BenchmarkId,
    context: &ReportContext,
    statistic: Statistic,
    distribution: &Distribution<f64>,
    estimate: &Estimate,
    noise_threshold: f64,
    size: Option<Size>,
) -> Child {
    let ci = estimate.confidence_interval;
    let (lb, ub) = (ci.lower_bound, ci.upper_bound);

    let start = lb - (ub - lb) / 9.;
    let end = ub + (ub - lb) / 9.;
    let (xs, ys) = kde::sweep(distribution, KDE_POINTS, Some((start, end)));
    let xs_ = Sample::new(&xs);

    let p = estimate.point_estimate;
    let n_p = xs.iter().enumerate().find(|&(_, &x)| x >= p).unwrap().0;
    let y_p = ys[n_p - 1] + (ys[n_p] - ys[n_p - 1]) / (xs[n_p] - xs[n_p - 1]) * (p - xs[n_p - 1]);

    let one = iter::repeat(1);
    let zero = iter::repeat(0);

    let start = xs.iter().enumerate().find(|&(_, &x)| x >= lb).unwrap().0;
    let end = xs
        .iter()
        .enumerate()
        .rev()
        .find(|&(_, &x)| x <= ub)
        .unwrap()
        .0;
    let len = end - start;

    let x_min = xs_.min();
    let x_max = xs_.max();

    let (fc_start, fc_end) = if noise_threshold < x_min || -noise_threshold > x_max {
        let middle = (x_min + x_max) / 2.;

        (middle, middle)
    } else {
        (
            if -noise_threshold < x_min {
                x_min
            } else {
                -noise_threshold
            },
            if noise_threshold > x_max {
                x_max
            } else {
                noise_threshold
            },
        )
    };

    let mut figure = Figure::new();

    figure
        .set(Font(DEFAULT_FONT))
        .set(size.unwrap_or(SIZE))
        .configure(Axis::LeftY, |a| a.set(Label("Density (a.u.)")))
        .configure(Key, |k| {
            k.set(Justification::Left)
                .set(Order::SampleText)
                .set(Position::Outside(Vertical::Top, Horizontal::Right))
        })
        .set(Title(format!(
            "{}: {}",
            escape_underscores(id.as_title()),
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

    let path = context.report_path(id, &format!("change/{}.svg", statistic));
    debug_script(&path, &figure);
    figure.set(Output(path)).draw().unwrap()
}

pub(crate) fn rel_distributions(
    id: &BenchmarkId,
    context: &ReportContext,
    _measurements: &MeasurementData,
    comparison: &ComparisonData,
    size: Option<Size>,
) -> Vec<Child> {
    comparison
        .relative_distributions
        .iter()
        .map(|(&statistic, distribution)| {
            rel_distribution(
                id,
                context,
                statistic,
                distribution,
                &comparison.relative_estimates[&statistic],
                comparison.noise_threshold,
                size,
            )
        })
        .collect::<Vec<_>>()
}
