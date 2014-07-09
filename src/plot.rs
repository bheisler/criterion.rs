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
use statistics::estimate::Estimate;
use units::{Unit,Time,Ratio};

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

pub fn points(sample: &[f64], dir: &Path) {
    fs::mkdirp(dir);

    let mut rng = rand::task_rng();

    Figure::new().
        set_output_file(dir.join("points.png")).
        set_title("Sample points").
        set_xlabel("Time (ns)").
        set_size(PNG_SIZE).
        plot(Points, sample.iter(), rng.gen_iter::<f64>(),
             [PointType(Circle)]).
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
        set_output_file(dir.join("boxplot.png")).
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

pub fn both_points(base: &[f64], new: &[f64], dir: &Path) {
    fs::mkdirp(dir);

    let mut rng = rand::task_rng();

    Figure::new().
        set_output_file(dir.join("points.png")).
        set_title("Sample points").
        set_xlabel("Time (ns)").
        set_size(PNG_SIZE).
        plot(Points, base.iter(), rng.gen_iter::<f64>(),
             [PointType(Circle), Title("Base")]).
        plot(Points, new.iter(), rng.gen_iter::<f64>(),
             [PointType(Circle), Title("New")]).
        draw();

}

pub fn both_pdfs(base: &[f64], new: &[f64], dir: &Path) {
    fs::mkdirp(dir);

    let (base_xs, base_ys) = math::kde(base);
    let (new_xs, new_ys) = math::kde(new);

    Figure::new().
        set_output_file(dir.join("pdfs.png")).
        set_title("Probability Density Functions").
        set_xlabel("Time (ns)").
        set_ylabel("Density (a.u.)").
        set_size(PNG_SIZE).
        plot(Lines, base_xs.iter(), base_ys.iter(), [Title("Base")]).
        plot(Lines, new_xs.iter(), new_ys.iter(), [Title("New")]).
        draw();

}

pub fn distribution(distribution: &[f64],
                    estimate: &Estimate,
                    dir: &Path,
                    statistic: &'static str,
                    unit: Unit) {
    let (xs, ys) = math::kde(distribution);
    let ys = ys.as_slice();
    let vertical = [ys.min(), ys.max()];

    let point = estimate.point();
    let (lb, ub) = (estimate.lower_bound(), estimate.upper_bound());

    let (xs, point, lb, ub) = match unit {
        Time => {
            // TODO Properly scale the time
            (xs, point, lb, ub)
        },
        Ratio => {(
            xs.iter().map(|y| y * 100.0).collect(),
            point * 100.0,
            lb * 100.0,
            ub * 100.0,
        )},
    };

    Figure::new().
        set_output_file(dir.join(format!("{}.png", statistic))).
        set_title("Probability Density Function").
        set_xlabel(match unit {
            Time => "Time (ns)",
            Ratio => "Relative change (%)"
        }).
        set_ylabel("Density (a.u.)").
        set_size(PNG_SIZE).
        plot(Lines, xs.iter(), ys.iter(), []).
        plot(Lines, [point, point].iter(), vertical.iter(), [Title("Point")]).
        plot(Lines,
             [lb, lb, ub, ub].iter(),
             vertical.iter().rev().chain(vertical.iter()),
             [Title("Confidence Interval")]).
        draw();
}
