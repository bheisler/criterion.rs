use simplot::prelude::*;
use stats::ConfidenceInterval;
use std::iter;
use std::num::Float;
use std::str;
use test::stats::Stats;

use estimate::{Estimate, Estimates};
use estimate::Statistic::Slope;
use kde;
use super::scale_time;
use super::{DARK_BLUE, DARK_RED, FONT, KDE_POINTS, LINEWIDTH, SIZE};

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
        set(FONT).
        set(Output(path)).
        set(SIZE).
        set(Title(id.to_string())).
        configure(Axis::BottomX, move |:a| a.
            configure(Grid::Major, |g| g.
                show()).
            set(Label(x_label))).
        configure(Axis::LeftY, |a| a.
            configure(Grid::Major, |g| g.
                show()).
            set(Label(format!("Total time ({}s)", prefix)))).
        configure(Key, |k| k.
            set(Justification::Left).
            set(Order::SampleText).
            set(Position::Inside(Vertical::Top, Horizontal::Left))).
        plot(FilledCurve {
            x: &[0., max_iters],
            y1: &[0., base_lb],
            y2: &[0., base_ub],
        }, |c| c.
            set(DARK_RED).
            set(Opacity(0.25))).
        plot(FilledCurve {
            x: &[0., max_iters],
            y1: &[0., new_lb],
            y2: &[0., new_ub],
        }, |c| c.
            set(DARK_BLUE).
            set(Opacity(0.25))).
        plot(Lines {
            x: &[0., max_iters],
            y: &[0., base_point],
        }, |c| c.
            set(DARK_RED).
            set(LINEWIDTH).
            set(Label("Base sample")).
            set(LineType::Solid)).
        plot(Lines {
            x: &[0., max_iters],
            y: &[0., new_point],
        }, |c| c.
            set(DARK_BLUE).
            set(LINEWIDTH).
            set(Label("New sample")).
            set(LineType::Solid)).
        draw().unwrap();

    assert_eq!(Some(""), gnuplot.wait_with_output().ok().as_ref().and_then(|po| {
        str::from_utf8(&*po.error).ok()
    }))
}

pub fn pdfs(base: &[f64], new: &[f64], id: &str) {
    let path = Path::new(format!(".criterion/{}/both/pdf.svg", id));

    let (base_xs, base_ys) = kde::sweep(base, KDE_POINTS, None);
    let (new_xs, new_ys) = kde::sweep(new, KDE_POINTS, None);

    let (scale, prefix) = scale_time([base_xs.max(), new_xs.max()].max());
    let rscale = scale.recip();
    let base_xs = base_xs.iter().map(|&x| x * scale);
    let base_ys = base_ys.iter().map(|&x| x * rscale);
    let new_xs = new_xs.iter().map(|&x| x * scale);
    let new_ys = new_ys.iter().map(|&x| x * rscale);
    let zeros = iter::repeat(0);

    let gnuplot = Figure::new().
        set(FONT).
        set(Output(path)).
        set(SIZE).
        set(Title(id.to_string())).
        configure(Axis::BottomX, |a| a.
            set(Label(format!("Average time ({}s)", prefix)))).
        configure(Axis::LeftY, |a| a.
            set(Label("Density (a.u.)"))).
        configure(Axis::RightY, |a| a.
            hide()).
        configure(Key, |k| k.
            set(Justification::Left).
            set(Order::SampleText).
            set(Position::Outside(Vertical::Top, Horizontal::Right))).
        plot(FilledCurve {
            x: base_xs,
            y1: base_ys,
            y2: zeros.clone(),
        }, |c| c.
            set(DARK_RED).
            set(Label("Base PDF")).
            set(Opacity(0.5))).
        plot(FilledCurve {
            x: new_xs,
            y1: new_ys,
            y2: zeros,
        }, |c| c.
            set(DARK_BLUE).
            set(Label("New PDF")).
            set(Opacity(0.5))).
        draw().unwrap();

    assert_eq!(Some(""), gnuplot.wait_with_output().ok().as_ref().and_then(|po| {
        str::from_utf8(&*po.error).ok()
    }))
}
