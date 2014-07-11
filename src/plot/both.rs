use simplot::Figure;
use simplot::option::{Title,PointType};
use simplot::plottype::{Lines,Points};
use simplot::pointtype::Circle;
use std::rand::Rng;
use std::rand;

use statistics::Sample;
use math;
use super::PNG_SIZE;

pub fn pdfs<V: Vector<f64>, W: Vector<f64>>(base: &Sample<V>, new: &Sample<W>, path: Path) {
    let (base_xs, base_ys) = math::kde(base.as_slice());
    let (new_xs, new_ys) = math::kde(new.as_slice());

    Figure::new().
        set_output_file(path).
        set_title("Probability Density Functions").
        set_xlabel("Time (ns)").
        set_ylabel("Density (a.u.)").
        set_size(PNG_SIZE).
        plot(Lines, base_xs.iter(), base_ys.iter(), [Title("Base")]).
        plot(Lines, new_xs.iter(), new_ys.iter(), [Title("New")]).
        draw();
}

pub fn points<V: Vector<f64>, W: Vector<f64>>(base: &Sample<V>, new: &Sample<W>, path: Path) {
    let mut rng = rand::task_rng();
    let base = base.as_slice();
    let new = new.as_slice();

    Figure::new().
        set_output_file(path).
        set_title("Sample points").
        set_xlabel("Time (ns)").
        set_size(PNG_SIZE).
        plot(Points, base.iter(), rng.gen_iter::<f64>(),
             [PointType(Circle), Title("Base")]).
        plot(Points, new.iter(), rng.gen_iter::<f64>(),
             [PointType(Circle), Title("New")]).
        draw();
}
