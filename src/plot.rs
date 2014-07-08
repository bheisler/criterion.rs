use simplot::Figure;
use simplot::option::{Title,PointType};
use simplot::plottype::{Lines,Points};
use simplot::pointtype::Circle;
use std::rand::Rng;
use std::rand;
use test::stats::Stats;

use fs;
use math;
use outliers::Outliers;

// XXX should the size of the image be configurable?
pub static PNG_SIZE: (uint, uint) = (1366, 768);

pub fn pdf(sample: &[f64], dir: &Path) {
    fs::mkdirp(dir);

    let (xs, ys) = math::kde(sample);
    let ys = ys.as_slice();
    let vertical = [ys.min(), ys.max()];
    let mean = sample.mean();
    let median = sample.median();
    let mean = [mean, mean];
    let median = [median, median];

    Figure::new().
        set_output_file(dir.join("pdf.png")).
        set_title("Probability Density Function").
        set_xlabel("Time (ns)").
        set_ylabel("Density (a.u.)").
        set_size(PNG_SIZE).
        plot(Lines, xs.iter(), ys.iter(), []).
        plot(Lines, mean.iter(), vertical.iter(), [Title("mean")]).
        plot(Lines, median.iter(), vertical.iter(), [Title("median")]).
        draw();
}

pub fn outliers(outliers: &Outliers, dir: &Path) {
    fs::mkdirp(dir);

    let mut rng = rand::task_rng();
    let (lost, lomt, himt, hist) = outliers.thresholds();
    let him = outliers.high_mild();
    let his = outliers.high_severe();
    let lom = outliers.low_mild();
    let los = outliers.low_severe();
    let normal = outliers.normal();
    let y = [1u, 0, 0, 1];
    let mild = lom.iter().chain(him.iter());
    let severe = los.iter().chain(his.iter());

    Figure::new().
        set_output_file(dir.join("outliers.png")).
        set_title("Classification of Outliers").
        set_xlabel("Time (ns)").
        set_size(PNG_SIZE).
        plot(Lines, [lomt, lomt, himt, himt].iter(), y.iter(), []).
        plot(Lines, [lost, lost, hist, hist].iter(), y.iter(), []).
        plot(Points, mild, rng.gen_iter::<f64>(),
             [PointType(Circle), Title("Mild")]).
        plot(Points, normal.iter(), rng.gen_iter::<f64>(),
             [PointType(Circle)]).
        plot(Points, severe, rng.gen_iter::<f64>(),
             [PointType(Circle), Title("Severe")]).
        draw();
}

pub fn both_points(old: &[f64], new: &[f64], dir: &Path) {
    fs::mkdirp(dir);

    let mut rng = rand::task_rng();

    Figure::new().
        set_output_file(dir.join("points.png")).
        set_title("Sample points").
        set_xlabel("Time (ns)").
        set_size(PNG_SIZE).
        plot(Points, old.iter(), rng.gen_iter::<f64>(),
             [PointType(Circle), Title("Old")]).
        plot(Points, new.iter(), rng.gen_iter::<f64>(),
             [PointType(Circle), Title("New")]).
        draw();

}

pub fn both_pdfs(old: &[f64], new: &[f64], dir: &Path) {
    fs::mkdirp(dir);

    let (old_xs, old_ys) = math::kde(old);
    let (new_xs, new_ys) = math::kde(new);

    Figure::new().
        set_output_file(dir.join("pdfs.png")).
        set_title("Probability Density Functions").
        set_xlabel("Time (ns)").
        set_ylabel("Density (a.u.)").
        set_size(PNG_SIZE).
        plot(Lines, old_xs.iter(), old_ys.iter(), [Title("Old")]).
        plot(Lines, new_xs.iter(), new_ys.iter(), [Title("New")]).
        draw();

}

pub fn bootstraps(distributions: &Vec<Vec<f64>>, dir: &Path) {
    let (mean, median) = match distributions.as_slice() {
        [ref mean, ref median] => (mean.as_slice(), median.as_slice()),
        _ => fail!("`distributions` should be vec![means, medians]"),
    };

    let (mean_xs, mean_ys) = math::kde(mean);
    Figure::new().
        set_output_file(dir.join("mean.png")).
        set_title("Bootstrapped Probability Density Function").
        set_xlabel("Ratio (%)").
        set_ylabel("Density (a.u.)").
        set_size(PNG_SIZE).
        plot(Lines, mean_xs.iter().map(|x| x * 100.0), mean_ys.iter(), []).
        draw();

    let (median_xs, median_ys) = math::kde(median);
    Figure::new().
        set_output_file(dir.join("median.png")).
        set_title("Bootstrapped Probability Density Function").
        set_xlabel("Ratio (%)").
        set_ylabel("Density (a.u.)").
        set_size(PNG_SIZE).
        plot(Lines, median_xs.iter().map(|x| x * 100.0), median_ys.iter(), []).
        draw();
}
