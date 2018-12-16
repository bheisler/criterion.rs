use std::process::Child;

use criterion_plot::prelude::*;
use estimate::Statistic;
use stats::bivariate::regression::Slope;

use super::*;
use report::{BenchmarkId, ComparisonData, MeasurementData, ReportContext};
use stats::bivariate::Data;

use {ConfidenceInterval, Estimate};

fn regression_figure(measurements: &MeasurementData, size: Option<Size>) -> Figure {
    let slope_estimate = &measurements.absolute_estimates[&Statistic::Slope];
    let point = Slope::fit(&measurements.data);
    let slope_dist = &measurements.distributions[&Statistic::Slope];
    let (lb, ub) =
        slope_dist.confidence_interval(slope_estimate.confidence_interval.confidence_level);

    let data = &measurements.data;
    let (max_iters, max_elapsed) = (data.x().max(), data.y().max());

    let exponent = (max_iters.log10() / 3.).floor() as i32 * 3;
    let x_scale = 10f64.powi(-exponent);

    let x_label = if exponent == 0 {
        "Iterations".to_owned()
    } else {
        format!("Iterations (x 10^{})", exponent)
    };

    let lb = lb * max_iters;
    let point = point.0 * max_iters;
    let ub = ub * max_iters;
    let max_iters = max_iters;

    let mut figure = Figure::new();
    figure
        .set(Font(DEFAULT_FONT))
        .set(size.unwrap_or(SIZE))
        .configure(Axis::BottomX, |a| {
            a.configure(Grid::Major, |g| g.show())
                .set(Label(x_label))
                .set(ScaleFactor(x_scale))
        })
        .configure(Axis::LeftY, |a| {
            let (y_scale, prefix) = scale_time(max_elapsed);
            a.configure(Grid::Major, |g| g.show())
                .set(Label(format!("Total sample time ({}s)", prefix)))
                .set(ScaleFactor(y_scale))
        })
        .plot(
            Points {
                x: data.x().as_ref(),
                y: data.y().as_ref(),
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
    figure
}

pub(crate) fn regression(
    id: &BenchmarkId,
    context: &ReportContext,
    measurements: &MeasurementData,
    size: Option<Size>,
) -> Child {
    let mut figure = regression_figure(measurements, size);
    figure.set(Title(escape_underscores(id.as_title())));
    figure.configure(Key, |k| {
        k.set(Justification::Left)
            .set(Order::SampleText)
            .set(Position::Inside(Vertical::Top, Horizontal::Left))
    });

    let path = context.report_path(id, "regression.svg");
    debug_script(&path, &figure);
    figure.set(Output(path)).draw().unwrap()
}

pub(crate) fn regression_small(
    id: &BenchmarkId,
    context: &ReportContext,
    measurements: &MeasurementData,
    size: Option<Size>,
) -> Child {
    let mut figure = regression_figure(measurements, size);
    figure.configure(Key, |k| k.hide());

    let path = context.report_path(id, "regression_small.svg");
    debug_script(&path, &figure);
    figure.set(Output(path)).draw().unwrap()
}

fn regression_comparison_figure(
    measurements: &MeasurementData,
    comparison: &ComparisonData,
    base_data: &Data<f64, f64>,
    size: Option<Size>,
) -> Figure {
    let data = &measurements.data;
    let max_iters = base_data.x().max().max(data.x().max());
    let max_elapsed = base_data.y().max().max(data.y().max());

    let (y_scale, prefix) = scale_time(max_elapsed);

    let exponent = (max_iters.log10() / 3.).floor() as i32 * 3;
    let x_scale = 10f64.powi(-exponent);

    let x_label = if exponent == 0 {
        "Iterations".to_owned()
    } else {
        format!("Iterations (x 10^{})", exponent)
    };

    let Estimate {
        confidence_interval:
            ConfidenceInterval {
                lower_bound: base_lb,
                upper_bound: base_ub,
                ..
            },
        point_estimate: base_point,
        ..
    } = comparison.base_estimates[&Statistic::Slope];

    let Estimate {
        confidence_interval:
            ConfidenceInterval {
                lower_bound: lb,
                upper_bound: ub,
                ..
            },
        point_estimate: point,
        ..
    } = comparison.base_estimates[&Statistic::Slope];

    let mut figure = Figure::new();
    figure
        .set(Font(DEFAULT_FONT))
        .set(size.unwrap_or(SIZE))
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
        .configure(Key, |k| {
            k.set(Justification::Left)
                .set(Order::SampleText)
                .set(Position::Inside(Vertical::Top, Horizontal::Left))
        })
        .plot(
            FilledCurve {
                x: &[0., max_iters],
                y1: &[0., base_lb * max_iters],
                y2: &[0., base_ub * max_iters],
            },
            |c| c.set(DARK_RED).set(Opacity(0.25)),
        )
        .plot(
            FilledCurve {
                x: &[0., max_iters],
                y1: &[0., lb * max_iters],
                y2: &[0., ub * max_iters],
            },
            |c| c.set(DARK_BLUE).set(Opacity(0.25)),
        )
        .plot(
            Lines {
                x: &[0., max_iters],
                y: &[0., base_point * max_iters],
            },
            |c| {
                c.set(DARK_RED)
                    .set(LINEWIDTH)
                    .set(Label("Base sample"))
                    .set(LineType::Solid)
            },
        )
        .plot(
            Lines {
                x: &[0., max_iters],
                y: &[0., point * max_iters],
            },
            |c| {
                c.set(DARK_BLUE)
                    .set(LINEWIDTH)
                    .set(Label("New sample"))
                    .set(LineType::Solid)
            },
        );
    figure
}

pub(crate) fn regression_comparison(
    id: &BenchmarkId,
    context: &ReportContext,
    measurements: &MeasurementData,
    comparison: &ComparisonData,
    base_data: &Data<f64, f64>,
    size: Option<Size>,
) -> Child {
    let mut figure = regression_comparison_figure(measurements, comparison, base_data, size);
    figure.set(Title(escape_underscores(id.as_title())));

    let path = context.report_path(id, "both/regression.svg");
    debug_script(&path, &figure);
    figure.set(Output(path)).draw().unwrap()
}

pub(crate) fn regression_comparison_small(
    id: &BenchmarkId,
    context: &ReportContext,
    measurements: &MeasurementData,
    comparison: &ComparisonData,
    base_data: &Data<f64, f64>,
    size: Option<Size>,
) -> Child {
    let mut figure = regression_comparison_figure(measurements, comparison, base_data, size);
    figure.configure(Key, |k| k.hide());

    let path = context.report_path(id, "relative_regression_small.svg");
    debug_script(&path, &figure);
    figure.set(Output(path)).draw().unwrap()
}
