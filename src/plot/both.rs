use simplot::Figure;
use simplot::option::{Title, PointType};
use simplot::plottype::{Lines, Points};
use simplot::pointtype::Circle;
use simplot::terminal::Svg;
use std::iter;
use test::stats::Stats;

use kde;
use super::scale_time;
use super::{FONT, KDE_POINTS, PLOT_SIZE};

pub fn regression(base: &[(u64, u64)], new: &[(f64, f64)], id: &str) {
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
    let base_elapsed: Vec<f64> = base.iter().map(|&(_, y)| y as f64 * y_scale).collect();
    let new_elapsed: Vec<f64> = new.iter().map(|&(_, y)| y * y_scale).collect();

    let exponent = (max_iters.log10() / 3.).floor() as i32 * 3;
    let x_scale = 10f64.powi(-exponent);
    let base_iters: Vec<f64> = base.iter().map(|&(x, _)| x as f64 * x_scale).collect();
    let new_iters: Vec<f64> = new.iter().map(|&(x, _)| x * x_scale).collect();

    let x_label = if exponent == 0 {
        "Iterations".to_string()
    } else {
        format!("Iterations (x 10^{})", exponent)
    };

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Linear regression", id)).
        set_xlabel(x_label).
        set_ylabel(format!("Total time ({}s)", prefix)).
        plot(Points, base_iters.iter(), base_elapsed.iter(), [PointType(Circle), Title("Base")]).
        plot(Points, new_iters.iter(), new_elapsed.iter(), [PointType(Circle), Title("New")]).
        draw();
}

pub fn pdfs(base: &[f64], new: &[f64], id: &str) {
    let path = Path::new(format!(".criterion/{}/both/pdf.svg", id));

    let (base_xs, base_ys) = kde::sweep(base, KDE_POINTS);
    let (new_xs, new_ys) = kde::sweep(new, KDE_POINTS);

    let (scale, prefix) =
        scale_time([base_xs.as_slice().max(), new_xs.as_slice().max()].as_slice().max());
    let rscale = scale.recip();
    let base_xs = base_xs.iter().map(|x| x * scale);
    let base_ys = base_ys.iter().map(|x| x * rscale);
    let new_xs = new_xs.iter().map(|x| x * scale);
    let new_ys = new_ys.iter().map(|x| x * rscale);

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Probability Density Functions", id)).
        set_xlabel(format!("Average time ({}s)", prefix)).
        set_ylabel("Density (a.u.)").
        plot(Lines, base_xs, base_ys, [Title("Base")]).
        plot(Lines, new_xs, new_ys, [Title("New")]).
        draw();
}

pub fn points(base: &[f64], new: &[f64], id: &str) {
    let path = Path::new(format!(".criterion/{}/both/sample.svg", id));

    let (scale, prefix) = scale_time([base.max(), new.max()].as_slice().max());
    let base = base.iter().map(|x| x * scale);
    let new = new.iter().map(|x| x * scale);

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Sample points", id)).
        set_xlabel(format!("Average time ({}s)", prefix)).
        plot(Points, base, iter::count(0u, 1), [PointType(Circle), Title("Base")]).
        plot(Points, new, iter::count(0u, 1), [PointType(Circle), Title("New")]).
        draw();
}
