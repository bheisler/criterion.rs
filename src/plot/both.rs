use std::path::PathBuf;
use std::{iter, str};

use simplot::prelude::*;
use stats::bivariate::Data;
use stats::univariate::Sample;

use {ConfidenceInterval, Estimate};
use estimate::Statistic::Slope;
use estimate::Estimates;
use kde;
use super::{debug_script, escape_underscores, scale_time, wait_on_gnuplot};
use super::{DARK_BLUE, DARK_RED, DEFAULT_FONT, KDE_POINTS, LINEWIDTH, SIZE};

#[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
pub(crate) fn regression(
    base_data: Data<f64, f64>,
    base_estimates: &Estimates,
    data: Data<f64, f64>,
    estimates: &Estimates,
    id: &str,
    path: String,
    size: Option<Size>,
    thumbnail_mode: bool,
) {
    let path = PathBuf::from(path);

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
    } = base_estimates[&Slope];

    let Estimate {
        confidence_interval:
            ConfidenceInterval {
                lower_bound: lb,
                upper_bound: ub,
                ..
            },
        point_estimate: point,
        ..
    } = estimates[&Slope];

    let mut figure = Figure::new();
    if !thumbnail_mode {
        figure.set(Title(escape_underscores(id)));
    }

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
                .set(Label(format!("Total time ({}s)", prefix)))
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
                y1: &[0., base_lb],
                y2: &[0., base_ub],
            },
            |c| c.set(DARK_RED).set(Opacity(0.25)),
        )
        .plot(
            FilledCurve {
                x: &[0., max_iters],
                y1: &[0., lb],
                y2: &[0., ub],
            },
            |c| c.set(DARK_BLUE).set(Opacity(0.25)),
        )
        .plot(
            Lines {
                x: &[0., max_iters],
                y: &[0., base_point],
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
                y: &[0., point],
            },
            |c| {
                c.set(DARK_BLUE)
                    .set(LINEWIDTH)
                    .set(Label("New sample"))
                    .set(LineType::Solid)
            },
        );
    debug_script(&path, &figure);
    let gnuplot = figure.set(Output(path)).draw().unwrap();

    wait_on_gnuplot(vec![gnuplot]);
}

pub fn pdfs(
    base_avg_times: &Sample<f64>,
    avg_times: &Sample<f64>,
    id: &str,
    path: String,
    size: Option<Size>,
    thumbnail_mode: bool,
) {
    let path = PathBuf::from(path);

    let (base_xs, base_ys) = kde::sweep(base_avg_times, KDE_POINTS, None);
    let (xs, ys) = kde::sweep(avg_times, KDE_POINTS, None);
    let base_xs_ = Sample::new(&base_xs);
    let xs_ = Sample::new(&xs);

    let (x_scale, prefix) = scale_time(base_xs_.max().max(xs_.max()));
    let y_scale = x_scale.recip();
    let zeros = iter::repeat(0);

    let mut figure = Figure::new();
    if !thumbnail_mode {
        figure.set(Title(escape_underscores(id)));
    }
    figure
        .set(Font(DEFAULT_FONT))
        .set(size.unwrap_or(SIZE))
        .configure(Axis::BottomX, |a| {
            a.set(Label(format!("Average time ({}s)", prefix)))
                .set(ScaleFactor(x_scale))
        })
        .configure(Axis::LeftY, |a| {
            a.set(Label("Density (a.u.)")).set(ScaleFactor(y_scale))
        })
        .configure(Axis::RightY, |a| a.hide())
        .configure(Key, |k| {
            if thumbnail_mode {
                k.hide();
            }
            k.set(Justification::Left)
                .set(Order::SampleText)
                .set(Position::Outside(Vertical::Top, Horizontal::Right))
        })
        .plot(
            FilledCurve {
                x: &*base_xs,
                y1: &*base_ys,
                y2: zeros.clone(),
            },
            |c| c.set(DARK_RED).set(Label("Base PDF")).set(Opacity(0.5)),
        )
        .plot(
            FilledCurve {
                x: &*xs,
                y1: &*ys,
                y2: zeros,
            },
            |c| c.set(DARK_BLUE).set(Label("New PDF")).set(Opacity(0.5)),
        );
    debug_script(&path, &figure);
    let gnuplot = figure.set(Output(path)).draw().unwrap();

    wait_on_gnuplot(vec![gnuplot]);
}
