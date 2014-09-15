use simplot::Figure;
use simplot::option::{Title, PointType};
use simplot::plottype::{Lines, Points};
use simplot::pointtype::Circle;
use simplot::terminal::Svg;
use stats::outliers::{Outliers, LowMild, LowSevere, HighMild, HighSevere};
use stats::regression::Slope;
use stats::{mean, median};
use std::io::fs::PathExtensions;
use std::iter;
use test::stats::Stats;

use estimate::{Distributions, Estimate, Estimates, Mean, Median};
use fs;
use kde;

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

// TODO These should be configurable
static FONT: &'static str = "Fantasque Sans Mono";
static KDE_POINTS: uint = 500;
static PLOT_SIZE: (uint, uint) = (1280, 720);

pub fn pdf(sample: &[f64], id: &str) {
    let path = Path::new(format!(".criterion/{}/new/pdf.svg", id));

    let (xs, ys) = kde::sweep(sample, KDE_POINTS);

    let (scale, prefix) = scale_time(xs.as_slice().max());
    let rscale = scale.recip();
    let xs = xs.move_iter().map(|x| x * scale).collect::<Vec<f64>>();
    let ys = ys.move_iter().map(|y| y * rscale).collect::<Vec<f64>>();

    let ys = ys.as_slice();
    let vertical = [ys.min(), ys.max()];
    let mean = mean(sample) * scale;
    let median = median(sample) * scale;
    let mean = [mean, mean];
    let median = [median, median];

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Probability Density Function", id.as_slice())).
        set_xlabel(format!("Average time ({}s)", prefix)).
        set_ylabel("Density (a.u.)").
        plot(Lines, xs.iter(), ys.iter(), []).
        plot(Lines, mean.iter(), vertical.iter(), [Title("Mean")]).
        plot(Lines, median.iter(), vertical.iter(), [Title("Median")]).
        draw();
}

pub fn regression(
    pairs: &[(f64, f64)],
    (lb, ub): (&Slope<f64>, &Slope<f64>),
    id: &str,
) {
    let path = Path::new(format!(".criterion/{}/new/regression.svg", id));

    let (mut min_iters, mut min_elapsed) = (0., 0.);
    let (mut max_iters, mut max_elapsed) = (0., 0.);

    for &(iters, elapsed) in pairs.iter() {
        if min_iters > iters {
            min_iters = iters;
        }

        if max_iters < iters {
            max_iters = iters;
        }

        if min_elapsed > elapsed {
            min_elapsed = elapsed;
        }

        if max_elapsed < elapsed {
            max_elapsed = elapsed;
        }
    }

    let (y_scale, prefix) = scale_time(max_elapsed);
    let elapsed: Vec<f64> = pairs.iter().map(|&(_, y)| y as f64 * y_scale).collect();

    let exponent = (max_iters.log10() / 3.).floor() as i32 * 3;
    let x_scale = 10f64.powi(-exponent);
    let iters: Vec<f64> = pairs.iter().map(|&(x, _)| x as f64 * x_scale).collect();

    let x_label = if exponent == 0 {
        "Iterations".to_string()
    } else {
        format!("Iterations (x 10^{})", exponent)
    };

    let x_min = min_iters * x_scale;
    let x_max = max_iters * x_scale;

    let alpha = lb.slope();
    let y_1 = alpha * max_iters * y_scale;
    let y_2 = alpha * min_iters * y_scale;

    let alpha = ub.slope();
    let y_3 = alpha * min_iters * y_scale;
    let y_4 = alpha * max_iters * y_scale;

    let xs = [x_max, x_min, x_min, x_max];
    let ys = [y_1, y_2, y_3, y_4];

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Linear regression", id.as_slice())).
        set_xlabel(x_label).
        set_ylabel(format!("Total time ({}s)", prefix)).
        plot(Points, iters.iter(), elapsed.iter(), [PointType(Circle)]).
        plot(Lines, xs.iter(), ys.iter(), [Title("Confidence Interval")]).
        draw();
}

pub fn slope(distribution: &[f64], (lb, point, ub): (f64, f64, f64), id: &str) {
    let path = Path::new(format!(".criterion/{}/new/slope.svg", id));

    let (xs, ys) = kde::sweep(distribution, KDE_POINTS);

    let (scale, prefix) = scale_time(xs.as_slice().max());
    let rscale = scale.recip();
    let xs = xs.move_iter().map(|x| x * scale).collect::<Vec<f64>>();
    let ys = ys.move_iter().map(|y| y * rscale).collect::<Vec<f64>>();
    let ys = ys.as_slice();

    let lb = lb * scale;
    let point = point * scale;
    let ub = ub * scale;
    let vertical = [ys.min(), ys.max()];

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Bootstrap distribution of the slope", id)).
        set_xlabel(format!("Average time ({}s)", prefix)).
        set_ylabel("Density (a.u.)").
        plot(Lines, xs.iter(), ys.iter(), []).
        plot(Lines, [point, point].iter(), vertical.iter(), [Title("Point estimate")]).
        plot(Lines,
             [lb, lb, ub, ub].iter(),
             vertical.iter().rev().chain(vertical.iter()),
             [Title("Confidence Interval")]).
        draw();
}

pub fn sample(sample: &[f64], id: &str) {
    let path = Path::new(format!(".criterion/{}/new/sample.svg", id));

    let (scale, prefix) = scale_time(sample.max());
    let sample = sample.iter().map(|x| x * scale).collect::<Vec<f64>>();

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Sample points", id.as_slice())).
        set_xlabel(format!("Average time ({}s)", prefix)).
        plot(Points, sample.iter(), iter::count(0u, 1), [PointType(Circle)]).
        draw();
}

pub fn abs_distributions(distributions: &Distributions, estimates: &Estimates, id: &str) {
    for (&statistic, distribution) in distributions.iter() {
        let estimate = estimates[statistic];

        let (xs, ys) = kde::sweep(distribution.as_slice(), KDE_POINTS);

        let (scale, prefix) = scale_time(xs.as_slice().max());
        let rscale = scale.recip();
        let xs = xs.move_iter().map(|x| x * scale).collect::<Vec<f64>>();
        let ys = ys.move_iter().map(|y| y * rscale).collect::<Vec<f64>>();

        let ys = ys.as_slice();
        let vertical = [ys.min(), ys.max()];

        let ci = estimate.confidence_interval;
        let p = estimate.point_estimate * scale;
        let (lb, ub) = (ci.lower_bound * scale, ci.upper_bound * scale);

        Figure::new().
            set_font(FONT).
            set_output_file(Path::new(format!(".criterion/{}/new/{}.svg", id, statistic))).
            set_size(PLOT_SIZE).
            set_terminal(Svg).
            set_title(format!("{}: Bootstrap distribution of the {}", id, statistic)).
            set_xlabel(format!("Average time ({}s)", prefix)).
            set_ylabel("Density (a.u.)").
            plot(Lines, xs.iter(), ys.iter(), []).
            plot(Lines, [p, p].iter(), vertical.iter(), [Title("Point estimate")]).
            plot(Lines,
                 [lb, lb, ub, ub].iter(),
                 vertical.iter().rev().chain(vertical.iter()),
                 [Title("Confidence Interval")]).
            draw();
    }
}

// TODO DRY: This is very similar to the `time_distributions` method
pub fn rel_distributions(
    distributions: &Distributions,
    estimates: &Estimates,
    id: &str,
) {
    for (&statistic, distribution) in distributions.iter() {
        let path = Path::new(format!(".criterion/{}/change/{}.svg", id, statistic));

        let (xs, ys) = kde::sweep(distribution.as_slice(), KDE_POINTS);
        let xs: Vec<f64> = xs.move_iter().map(|x| x * 100.0).collect();
        let ys = ys.as_slice();
        let vertical = [ys.min(), ys.max()];

        let estimate = estimates[statistic];
        let point = estimate.point_estimate * 100.0;
        let ci = estimate.confidence_interval;
        let (lb, ub) = (ci.lower_bound * 100.0, ci.upper_bound * 100.0);

        Figure::new().
            set_font(FONT).
            set_output_file(path).
            set_size(PLOT_SIZE).
            set_terminal(Svg).
            set_title(format!("{}: Bootstrap distribution of the {}", id, statistic)).
            set_xlabel("Relative change (%)").
            set_ylabel("Density (a.u.)").
            plot(Lines, xs.iter(), ys.iter(), []).
            plot(Lines, [point, point].iter(), vertical.iter(), [Title("Point estimate")]).
            plot(Lines,
                 [lb, lb, ub, ub].iter(),
                 vertical.iter().rev().chain(vertical.iter()),
                 [Title("Confidence Interval")]).
            draw();
    }
}

pub fn t_test(t: f64, distribution: &[f64], id: &str) {
    let path = Path::new(format!(".criterion/{}/change/t-test.svg", id));

    let (xs, ys) = kde::sweep(distribution, KDE_POINTS);
    let ys = ys.as_slice();
    let vertical = [ys.min(), ys.max()];

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Welch's t test", id)).
        set_xlabel("t score").
        set_ylabel("Density").
        plot(Lines, xs.iter(), ys.iter(), [Title("t distribution")]).
        plot(Lines, [t, t].iter(), vertical.iter(), [Title("t statistic")]).
        draw();
}

pub fn outliers(outliers: &Outliers<f64>, times: &[f64], id: &str) {
    let path = Path::new(format!(".criterion/{}/new/outliers.svg", id));

    let (mut lost, mut lomt, mut himt, mut hist) = outliers.fences;
    let (nlos, nlom, nnao, nhim, nhis) = outliers.count;

    let (scale, prefix) = scale_time(times.max());
    let mut mild = Vec::with_capacity(nlom + nhim);
    let mut severe = Vec::with_capacity(nlos + nhis);
    let mut normal = Vec::with_capacity(nnao);

    for (time, (&label, i)) in
        times.iter().map(|x| x * scale).zip(outliers.labels.iter().zip(iter::count(0u, 1)))
    {
        match label {
            HighSevere | LowSevere => severe.push((i, time)),
            LowMild | HighMild => mild.push((i, time)),
            _ => normal.push((i, time)),

        }
    }
    himt *= scale;
    hist *= scale;
    lomt *= scale;
    lost *= scale;

    let n = times.len();
    let y = [n, 0, 0, n];

    Figure::new().
        set_font(FONT).
        set_output_file(path).
        set_size(PLOT_SIZE).
        set_terminal(Svg).
        set_title(format!("{}: Classification of outliers", id)).
        set_xlabel(format!("Average time ({}s)", prefix)).
        plot(Lines, [lomt, lomt, himt, himt].iter(), y.iter(), []).
        plot(Lines, [lost, lost, hist, hist].iter(), y.iter(), []).
        plot(Points, mild.iter().map(|x| x.val1()), mild.iter().map(|x| x.val0()),
             [PointType(Circle), Title("Mild")]).
        plot(Points, normal.iter().map(|x| x.val1()), normal.iter().map(|x| x.val0()),
             [PointType(Circle)]).
        plot(Points, severe.iter().map(|x| x.val1()), severe.iter().map(|x| x.val0()),
             [PointType(Circle), Title("Severe")]).
        draw();
}

pub fn summarize(id: &str) {
    let dir = Path::new(".criterion").join(id);
    let contents = fs::ls(&dir);

    // TODO Specially handle inputs that can be parsed as `int`s
    // TODO Need better way to handle log scale triggering
    for &sample in ["new", "base"].iter() {
        for &statistic in [Mean, Median].iter() {
            let mut estimates_pairs = Vec::new();
            for entry in contents.iter().filter(|entry| {
                entry.is_dir() && entry.filename_str() != Some("summary")
            }) {
                let input = entry.filename_str().unwrap();
                let path = entry.join(sample).join("estimates.json");
                match Estimate::load(&path) {
                    Some(estimates) => estimates_pairs.push((estimates, input)),
                    _ => {}
                }
            }

            if estimates_pairs.is_empty() {
                continue;
            }

            estimates_pairs.sort_by(|&(ref a, _), &(ref b, _)| {
                let a = a[statistic].point_estimate;
                let b = b[statistic].point_estimate;
                b.partial_cmp(&a).unwrap()
            });

            let inputs = estimates_pairs.iter().map(|&(_, input)| input);
            let points = estimates_pairs.iter().map(|&(ref estimates, _)| {
                estimates[statistic].point_estimate
            }).collect::<Vec<f64>>();
            let lbs = estimates_pairs.iter().map(|&(ref estimates, _)| {
                estimates[statistic].confidence_interval.lower_bound
            }).collect::<Vec<f64>>();
            let ubs = estimates_pairs.iter().map(|&(ref estimates, _)| {
                estimates[statistic].confidence_interval.upper_bound
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
                set_xlabel(format!("Average time ({}s)", prefix)).
                xerrorbars(
                    points.iter(), iter::count(0u, 1), lbs, ubs, [Title("Confidence Interval")]).
                draw();
        }
    }
}
