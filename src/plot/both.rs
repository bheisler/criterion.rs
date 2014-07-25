use simplot::Figure;
use simplot::option::{Title,PointType};
use simplot::plottype::{Lines,Points};
use simplot::pointtype::Circle;
use std::rand::Rng;
use std::rand;
use test::stats::Stats;

use math;
use statistics::Sample;
use super::PNG_SIZE;
use super::scale_time;

pub fn pdfs<V: Vector<f64>, W: Vector<f64>>(base: &Sample<V>, new: &Sample<W>, path: Path) {
    let (base_xs, base_ys) = math::kde(base.as_slice());
    let (new_xs, new_ys) = math::kde(new.as_slice());

    let (scale, prefix) =
        scale_time([base_xs.as_slice().max(), new_xs.as_slice().max()].as_slice().max());
    let rscale = scale.recip();
    let base_xs = base_xs.iter().map(|x| x * scale);
    let base_ys = base_ys.iter().map(|x| x * rscale);
    let new_xs = new_xs.iter().map(|x| x * scale);
    let new_ys = new_ys.iter().map(|x| x * rscale);

    Figure::new().
        set_output_file(path).
        set_title("Probability Density Functions").
        set_xlabel(format!("Time ({}s)", prefix)).
        set_ylabel("Density (a.u.)").
        set_size(PNG_SIZE).
        plot(Lines, base_xs, base_ys, [Title("Base")]).
        plot(Lines, new_xs, new_ys, [Title("New")]).
        draw();
}

pub fn points<V: Vector<f64>, W: Vector<f64>>(base: &Sample<V>, new: &Sample<W>, path: Path) {
    let mut rng = rand::task_rng();
    let base = base.as_slice();
    let new = new.as_slice();

    let (scale, prefix) = scale_time([base.max(), new.max()].as_slice().max());
    let base = base.iter().map(|x| x * scale);
    let new = new.iter().map(|x| x * scale);

    Figure::new().
        set_output_file(path).
        set_title("Sample points").
        set_xlabel(format!("Time ({}s)", prefix)).
        set_size(PNG_SIZE).
        plot(Points, base, rng.gen_iter::<f64>(), [PointType(Circle), Title("Base")]).
        plot(Points, new, rng.gen_iter::<f64>(), [PointType(Circle), Title("New")]).
        draw();
}
