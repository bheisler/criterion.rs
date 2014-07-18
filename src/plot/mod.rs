use simplot::Figure;
use simplot::option::{Title,PointType};
use simplot::plottype::{Lines,Points};
use simplot::pointtype::Circle;
use std::iter;
use std::rand::Rng;
use std::rand;
use test::stats::Stats;

use fs;
use math;
use outliers::Outliers;
use statistics::{Distributions,Estimates,Mean,Median,Sample};

pub mod both;

// TODO Scale the time axis

// TODO This should be configurable
static PNG_SIZE: (uint, uint) = (1366, 768);

pub fn pdf<V: Vector<f64>>(s: &Sample<V>, path: Path) {
    let (xs, ys) = math::kde(s.as_slice());

    let ys = ys.as_slice();
    let vertical = [ys.min(), ys.max()];
    let mean = s.compute(Mean);
    let median = s.compute(Median);
    let mean = [mean, mean];
    let median = [median, median];

    Figure::new().
        set_output_file(path).
        set_title("Probability Density Function").
        set_xlabel("Time (ns)").
        set_ylabel("Density (a.u.)").
        set_size(PNG_SIZE).
        plot(Lines, xs.iter(), ys.iter(), []).
        plot(Lines, mean.iter(), vertical.iter(), [Title("Mean")]).
        plot(Lines, median.iter(), vertical.iter(), [Title("Median")]).
        draw();
}

pub fn sample<V: Vector<f64>>(s: &Sample<V>, path: Path) {
    let mut rng = rand::task_rng();
    let sample = s.as_slice();

    Figure::new().
        set_output_file(path).
        set_title("Sample points").
        set_xlabel("Time (ns)").
        set_size(PNG_SIZE).
        plot(Points, sample.iter(), rng.gen_iter::<f64>(), [PointType(Circle)]).
        draw();
}

pub fn time_distributions(d: &Distributions, e: &Estimates, dir: &Path) {
    for (&statistic, distribution) in d.iter() {
        let (xs, ys) = math::kde(distribution.as_slice());
        let ys = ys.as_slice();
        let vertical = [ys.min(), ys.max()];

        let estimate = e.get(statistic);
        let point = estimate.point_estimate();
        let ci = estimate.confidence_interval();
        let (lb, ub) = (ci.lower_bound(), ci.upper_bound());

        Figure::new().
            set_output_file(dir.join(format!("{}.png", statistic))).
            set_title("Bootstrap distribution").
            set_xlabel("Time (ns)").
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
}

// TODO DRY: This is very similar to the `time_distributions` method
pub fn ratio_distributions(d: &Distributions, e: &Estimates, dir: &Path) {
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
            set_output_file(dir.join(format!("{}.png", statistic))).
            set_title("Bootstrap distribution").
            set_xlabel("Relative change (%)").
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
}

pub fn outliers(outliers: &Outliers, path: Path) {
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
        set_output_file(path).
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

pub fn summarize(dir: &Path) {
    let contents = fs::ls(dir);

    // TODO Specially handle inputs that can be parsed as `int`s
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
            });
            let ubs = estimates_pairs.iter().map(|&(ref estimates, _)| {
                estimates.get(statistic).confidence_interval().upper_bound()
            });

            fs::mkdirp(&dir.join(format!("summary/{}", sample)));
            Figure::new().
                set_logscale((points.last().unwrap() / points[0] > 50.0, false)).
                set_output_file(dir.join(format!("summary/{}/{}s.png", sample, statistic))).
                set_size(PNG_SIZE).
                set_title(format!("{}", statistic)).
                set_ylabel("Input").
                set_ytics(inputs, iter::count(0u, 1)).
                set_yrange((-0.5, estimates_pairs.len() as f64 - 0.5)).
                set_xlabel("Time (ns)").
                xerrorbars(
                    points.iter(), iter::count(0u, 1), lbs, ubs, [Title("Confidence Interval")]).
                draw();
        }
    }
}
