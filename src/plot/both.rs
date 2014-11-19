use simplot::curve::Style::{Lines};
use simplot::key::{Horizontal, Justification, Order, Position, Vertical};
use simplot::{Axis, Figure, Grid, LineType};
use stats::ConfidenceInterval;
use std::iter::Repeat;
use std::num::Float;
use std::str;
use test::stats::Stats;

use estimate::{Estimate, Estimates};
use estimate::Statistic::Slope;
use kde;
use super::scale_time;
use super::{DARK_BLUE, DARK_RED};
use super::{FONT, KDE_POINTS, PLOT_SIZE};

pub fn regression(
    base: &[(u64, u64)],
    base_estimates: &Estimates,
    new: &[(f64, f64)],
    new_estimates: &Estimates,
    id: &str,
) {
    let path = Path::new(format!(".criterion/{}/both/regression.svg", id));

    let (mut max_iters, mut max_elapsed) = (0., 0.);

    for &(iters, elapsed) in base.iter() {
        if max_iters < iters as f64 {
            max_iters = iters as f64;
        }

        if max_elapsed < elapsed as f64 {
            max_elapsed = elapsed as f64;
        }
    }

    for &(iters, elapsed) in new.iter() {
        if max_iters < iters {
            max_iters = iters;
        }

        if max_elapsed < elapsed {
            max_elapsed = elapsed;
        }
    }

    let (y_scale, prefix) = scale_time(max_elapsed);

    let exponent = (max_iters.log10() / 3.).floor() as i32 * 3;
    let x_scale = 10f64.powi(-exponent);

    let x_label = if exponent == 0 {
        "Iterations".to_string()
    } else {
        format!("Iterations (x 10^{})", exponent)
    };

    let Estimate {
        confidence_interval: ConfidenceInterval { lower_bound: lb, upper_bound: ub, .. },
        point_estimate: point,
        ..
    } = base_estimates[Slope];
    let base_lb = lb * max_iters * y_scale;
    let base_point = point * max_iters * y_scale;
    let base_ub = ub * max_iters * y_scale;

    let Estimate {
        confidence_interval: ConfidenceInterval { lower_bound: lb, upper_bound: ub, .. },
        point_estimate: point,
        ..
    } = new_estimates[Slope];
    let new_lb = lb * max_iters * y_scale;
    let new_point = point * max_iters * y_scale;
    let new_ub = ub * max_iters * y_scale;

    let max_iters = max_iters * x_scale;

    let gnuplot = Figure::new().
        font(FONT).
        output(path).
        size(PLOT_SIZE).
        title(id.to_string()).
        axis(Axis::BottomX, |a| a.
            grid(Grid::Major, |g| g.
                show()).
             // FIXME (unboxed closures) remove cloning
            label(x_label.to_string())).
        axis(Axis::LeftY, |a| a.
            grid(Grid::Major, |g| g.
                show()).
            label(format!("Total time ({}s)", prefix))).
        key(|k| k.
            justification(Justification::Left).
            order(Order::SampleText).
            position(Position::Inside(Vertical::Top, Horizontal::Left))).
        filled_curve([0., max_iters].iter(), [0., base_lb].iter(), [0., base_ub].iter(), |c| c.
            color(DARK_RED).
            opacity(0.25)).
        filled_curve([0., max_iters].iter(), [0., new_lb].iter(), [0., new_ub].iter(), |c| c.
            color(DARK_BLUE).
            opacity(0.25)).
        curve(Lines, [0., max_iters].iter(), [0., base_point].iter(), |c| c.
            color(DARK_RED).
            label("Base sample").
            line_type(LineType::Solid).
            linewidth(2.)).
        curve(Lines, [0., max_iters].iter(), [0., new_point].iter(), |c| c.
            color(DARK_BLUE).
            label("New sample").
            line_type(LineType::Solid).
            linewidth(2.)).
        draw().unwrap();

    assert_eq!(Some(""), gnuplot.wait_with_output().ok().as_ref().and_then(|po| {
        str::from_utf8(po.error[])
    }))
}

pub fn pdfs(base: &[f64], new: &[f64], id: &str) {
    let path = Path::new(format!(".criterion/{}/both/pdf.svg", id));

    let (base_xs, base_ys) = kde::sweep(base, KDE_POINTS, None);
    let (new_xs, new_ys) = kde::sweep(new, KDE_POINTS, None);

    let (scale, prefix) = scale_time([base_xs[].max(), new_xs[].max()][].max());
    let rscale = scale.recip();
    let base_xs = base_xs.iter().map(|&x| x * scale);
    let base_ys = base_ys.iter().map(|&x| x * rscale);
    let new_xs = new_xs.iter().map(|&x| x * scale);
    let new_ys = new_ys.iter().map(|&x| x * rscale);
    let zeros = Repeat::new(0u);

    let gnuplot = Figure::new().
        font(FONT).
        output(path).
        size(PLOT_SIZE).
        title(id.to_string()).
        axis(Axis::BottomX, |a| a.
            label(format!("Average time ({}s)", prefix))).
        axis(Axis::LeftY, |a| a.
            label("Density (a.u.)")).
        axis(Axis::RightY, |a| a.
            hide()).
        key(|k| k.
            justification(Justification::Left).
            order(Order::SampleText).
            position(Position::Outside(Vertical::Top, Horizontal::Right))).
        filled_curve(base_xs, base_ys, zeros, |c| c.
            color(DARK_RED).
            label("Base PDF").
            opacity(0.5)).
        filled_curve(new_xs, new_ys, zeros, |c| c.
            color(DARK_BLUE).
            label("New PDF").
            opacity(0.5)).
        draw().unwrap();

    assert_eq!(Some(""), gnuplot.wait_with_output().ok().as_ref().and_then(|po| {
        str::from_utf8(po.error[])
    }))
}
