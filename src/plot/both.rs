use simplot::Figure;
use simplot::option::{Title, PointType};
use simplot::plottype::{Lines, Points};
use simplot::pointtype::Circle;
use simplot::terminal::Svg;
use std::rand::Rng;
use std::rand;
use test::stats::Stats;

use kde;
use super::scale_time;
use super::{FONT, KDE_POINTS, PLOT_SIZE};

pub fn pdfs(base: &[f64], new: &[f64], path: Path, id: &str) {
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
        set_xlabel(format!("Time ({}s)", prefix)).
        set_ylabel("Density (a.u.)").
        plot(Lines, base_xs, base_ys, [Title("Base")]).
        plot(Lines, new_xs, new_ys, [Title("New")]).
        draw();
}

pub fn points(base: &[f64], new: &[f64], path: Path, id: &str) {
    let mut rng = rand::task_rng();

    let (scale, prefix) = scale_time([base.max(), new.max()].as_slice().max());
    let base = base.iter().map(|x| x * scale);
    let new = new.iter().map(|x| x * scale);

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Sample points", id)).
        set_xlabel(format!("Time ({}s)", prefix)).
        plot(Points, base, rng.gen_iter::<f64>(), [PointType(Circle), Title("Base")]).
        plot(Points, new, rng.gen_iter::<f64>(), [PointType(Circle), Title("New")]).
        draw();
}
