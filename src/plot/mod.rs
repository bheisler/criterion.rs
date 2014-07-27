use simplot::Figure;
use simplot::option::{Title,PointType};
use simplot::plottype::{Lines,Points};
use simplot::pointtype::Circle;
use simplot::terminal::Svg;
use std::iter;
use std::rand::Rng;
use std::rand;
use test::stats::Stats;

use fs;
use math;
use outliers::Outliers;
use statistics::{Distributions,Estimates,Mean,Median,Sample};

pub mod both;

fn scale_time(ns: f64) -> (f64, &'static str) {
    if ns < 10f64.powi(0) {
        (10f64.powi(3), "p")
    } else if ns < 10f64.powi(3) {
        (10f64.powi(0), "n")
    } else if ns < 10f64.powi(6) {
        (10f64.powi(-3), "u")
    } else if ns < 10f64.powi(9) {
        (10f64.powi(-6), "m")
    } else {
        (10f64.powi(-9), "")
    }
}

// TODO This should be configurable
static PLOT_SIZE: (uint, uint) = (880, 495);
static FONT: &'static str = "Fantasque Sans Mono";

pub fn pdf<S: Str, V: Vector<f64>>(s: &Sample<V>, path: Path, id: S) {
    let (xs, ys) = math::kde(s.as_slice());

    let (scale, prefix) = scale_time(xs.as_slice().max());
    let rscale = scale.recip();
    let xs = xs.move_iter().map(|x| x * scale).collect::<Vec<f64>>();
    let ys = ys.move_iter().map(|y| y * rscale).collect::<Vec<f64>>();

    let ys = ys.as_slice();
    let vertical = [ys.min(), ys.max()];
    let mean = s.compute(Mean) * scale;
    let median = s.compute(Median) * scale;
    let mean = [mean, mean];
    let median = [median, median];

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Probability Density Function", id.as_slice())).
        set_xlabel(format!("Time ({}s)", prefix)).
        set_ylabel("Density (a.u.)").
        plot(Lines, xs.iter(), ys.iter(), []).
        plot(Lines, mean.iter(), vertical.iter(), [Title("Mean")]).
        plot(Lines, median.iter(), vertical.iter(), [Title("Median")]).
        draw();
}

pub fn sample<S: Str, V: Vector<f64>>(s: &Sample<V>, path: Path, id: S) {
    let mut rng = rand::task_rng();
    let sample = s.as_slice();

    let (scale, prefix) = scale_time(sample.max());
    let sample = sample.iter().map(|x| x * scale).collect::<Vec<f64>>();

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Sample points", id.as_slice())).
        set_xlabel(format!("Time ({}s)", prefix)).
        plot(Points, sample.iter(), rng.gen_iter::<f64>(), [PointType(Circle)]).
        draw();
}

pub fn time_distributions(d: &Distributions, e: &Estimates, dir: &Path, id: &str) {
    for (&statistic, distribution) in d.iter() {
        let (xs, ys) = math::kde(distribution.as_slice());

        let (scale, prefix) = scale_time(xs.as_slice().max());
        let rscale = scale.recip();
        let xs = xs.move_iter().map(|x| x * scale).collect::<Vec<f64>>();
        let ys = ys.move_iter().map(|y| y * rscale).collect::<Vec<f64>>();

        let ys = ys.as_slice();
        let vertical = [ys.min(), ys.max()];

        let estimate = e.get(statistic);
        let point = estimate.point_estimate() * scale;
        let ci = estimate.confidence_interval();
        let (lb, ub) = (ci.lower_bound() * scale, ci.upper_bound() * scale);

        Figure::new().
            set_font(FONT).
            set_output_file(dir.join(format!("{}.svg", statistic))).
            set_size(PLOT_SIZE).
            set_terminal(Svg).
            set_title(format!("{}: Bootstrap distribution of the {}", id, statistic)).
            set_xlabel(format!("Time ({}s)", prefix)).
            set_ylabel("Density (a.u.)").
            plot(Lines, xs.iter(), ys.iter(), []).
            plot(Lines, [point, point].iter(), vertical.iter(), [Title("Point")]).
            plot(Lines,
                 [lb, lb, ub, ub].iter(),
                 vertical.iter().rev().chain(vertical.iter()),
                 [Title("Confidence Interval")]).
            draw();
    }
}

// TODO DRY: This is very similar to the `time_distributions` method
pub fn ratio_distributions(d: &Distributions, e: &Estimates, dir: &Path, id: &str) {
    for (&statistic, distribution) in d.iter() {
        let (xs, ys) = math::kde(distribution.as_slice());
        let xs: Vec<f64> = xs.iter().map(|x| x * 100.0).collect();
        let ys = ys.as_slice();
        let vertical = [ys.min(), ys.max()];

        let estimate = e.get(statistic);
        let point = estimate.point_estimate() * 100.0;
        let ci = estimate.confidence_interval();
        let (lb, ub) = (ci.lower_bound() * 100.0, ci.upper_bound() * 100.0);

        Figure::new().
            set_font(FONT).
            set_output_file(dir.join(format!("{}.svg", statistic))).
            set_size(PLOT_SIZE).
            set_terminal(Svg).
            set_title(format!("{}: Bootstrap distribution of the {}", id, statistic)).
            set_xlabel("Relative change (%)").
            set_ylabel("Density (a.u.)").
            plot(Lines, xs.iter(), ys.iter(), []).
            plot(Lines, [point, point].iter(), vertical.iter(), [Title("Point")]).
            plot(Lines,
                 [lb, lb, ub, ub].iter(),
                 vertical.iter().rev().chain(vertical.iter()),
                 [Title("Confidence Interval")]).
            draw();
    }
}

pub fn outliers(outliers: &Outliers, path: Path, id: &str) {
    let mut rng = rand::task_rng();

    let (mut lost, mut lomt, mut himt, mut hist) = outliers.thresholds();
    let him = outliers.high_mild();
    let his = outliers.high_severe();
    let lom = outliers.low_mild();
    let los = outliers.low_severe();
    let normal = outliers.normal();

    let (scale, prefix) = scale_time(if his.is_empty() { hist } else { his.max() });
    let him = him.iter().map(|x| x * scale);
    let his = his.iter().map(|x| x * scale);
    let lom = lom.iter().map(|x| x * scale);
    let los = los.iter().map(|x| x * scale);
    let normal = normal.iter().map(|x| x * scale);
    himt *= scale;
    hist *= scale;
    lomt *= scale;
    lost *= scale;

    let mild = lom.chain(him);
    let severe = los.chain(his);

    let y = [1u, 0, 0, 1];

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Classification of outliers", id)).
        set_xlabel(format!("Time ({}s)", prefix)).
        plot(Lines, [lomt, lomt, himt, himt].iter(), y.iter(), []).
        plot(Lines, [lost, lost, hist, hist].iter(), y.iter(), []).
        plot(Points, mild, rng.gen_iter::<f64>(),
             [PointType(Circle), Title("Mild")]).
        plot(Points, normal, rng.gen_iter::<f64>(),
             [PointType(Circle)]).
        plot(Points, severe, rng.gen_iter::<f64>(),
             [PointType(Circle), Title("Severe")]).
        draw();
}

pub fn summarize(dir: &Path, id: &str) {
    let contents = fs::ls(dir);

    // TODO Specially handle inputs that can be parsed as `int`s
    // TODO Need better way to handle log scale triggering
    for &sample in ["new", "base"].iter() {
        for &statistic in [Mean, Median].iter() {
            let mut estimates_pairs = Vec::new();
            for entry in contents.iter().filter(|entry| {
                entry.is_dir() && entry.filename_str() != Some("summary")
            }) {
                let input = entry.filename_str().unwrap();
                let path = entry.join(sample).join("bootstrap/estimates.json");
                match Estimates::load(&path) {
                    Some(estimates) => estimates_pairs.push((estimates, input)),
                    _ => {}
                }
            }

            if estimates_pairs.is_empty() {
                continue;
            }

            estimates_pairs.sort_by(|&(ref a, _), &(ref b, _)| {
                let a = a.get(statistic).point_estimate();
                let b = b.get(statistic).point_estimate();
                b.partial_cmp(&a).unwrap()
            });

            let inputs = estimates_pairs.iter().map(|&(_, input)| input);
            let points = estimates_pairs.iter().map(|&(ref estimates, _)| {
                estimates.get(statistic).point_estimate()
            }).collect::<Vec<f64>>();
            let lbs = estimates_pairs.iter().map(|&(ref estimates, _)| {
                estimates.get(statistic).confidence_interval().lower_bound()
            }).collect::<Vec<f64>>();
            let ubs = estimates_pairs.iter().map(|&(ref estimates, _)| {
                estimates.get(statistic).confidence_interval().upper_bound()
            }).collect::<Vec<f64>>();

            let (scale, prefix) = scale_time(ubs.as_slice().max());
            let points = points.iter().map(|x| x * scale).collect::<Vec<f64>>();
            let lbs = lbs.iter().map(|x| x * scale);
            let ubs = ubs.iter().map(|x| x * scale);

            fs::mkdirp(&dir.join(format!("summary/{}", sample)));
            Figure::new().
                set_font(FONT).
                set_logscale((points[0] / *points.last().unwrap() > 50.0, false)).
                set_output_file(dir.join(format!("summary/{}/{}s.svg", sample, statistic))).
                set_size(PLOT_SIZE).
                set_terminal(Svg).
                set_title(format!("{}: Estimates of the {}s", id, statistic)).
                set_ylabel("Input").
                set_ytics(inputs, iter::count(0u, 1)).
                set_yrange((-0.5, estimates_pairs.len() as f64 - 0.5)).
                set_xlabel(format!("Time ({}s)", prefix)).
                xerrorbars(
                    points.iter(), iter::count(0u, 1), lbs, ubs, [Title("Confidence Interval")]).
                draw();
        }
    }
}
