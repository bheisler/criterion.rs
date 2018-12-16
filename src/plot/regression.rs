use std::process::Child;

use criterion_plot::prelude::*;
use estimate::Statistic;
use stats::bivariate::regression::Slope;

use super::*;
use report::{BenchmarkId, MeasurementData, ReportContext};

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
