use super::{debug_script, escape_underscores};
use super::{DARK_BLUE, DEFAULT_FONT, KDE_POINTS, LINEWIDTH, POINT_SIZE, SIZE};
use crate::kde;
use crate::measurement::ValueFormatter;
use crate::report::{BenchmarkId, ValueType};
use crate::stats::univariate::Sample;
use crate::AxisScale;
use criterion_plot::prelude::*;
use itertools::Itertools;
use std::cmp::Ordering;
use std::path::PathBuf;
use std::process::Child;

const NUM_COLORS: usize = 8;
static COMPARISON_COLORS: [Color; NUM_COLORS] = [
    Color::Rgb(178, 34, 34),
    Color::Rgb(46, 139, 87),
    Color::Rgb(0, 139, 139),
    Color::Rgb(255, 215, 0),
    Color::Rgb(0, 0, 139),
    Color::Rgb(220, 20, 60),
    Color::Rgb(139, 0, 139),
    Color::Rgb(0, 255, 127),
];

impl AxisScale {
    fn to_gnuplot(self) -> Scale {
        match self {
            AxisScale::Linear => Scale::Linear,
            AxisScale::Logarithmic => Scale::Logarithmic,
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::explicit_counter_loop))]
pub fn line_comparison(
    formatter: &dyn ValueFormatter,
    title: &str,
    all_curves: &[&(&BenchmarkId, Vec<f64>)],
    path: &str,
    value_type: ValueType,
    axis_scale: AxisScale,
) -> Child {
    let path = PathBuf::from(path);
    let mut f = Figure::new();

    let input_suffix = match value_type {
        ValueType::Bytes => " Size (Bytes)",
        ValueType::Elements => " Size (Elements)",
        ValueType::Value => "",
    };

    f.set(Font(DEFAULT_FONT))
        .set(SIZE)
        .configure(Key, |k| {
            k.set(Justification::Left)
                .set(Order::SampleText)
                .set(Position::Outside(Vertical::Top, Horizontal::Right))
        })
        .set(Title(format!("{}: Comparison", escape_underscores(title))))
        .configure(Axis::BottomX, |a| {
            a.set(Label(format!("Input{}", input_suffix)))
                .set(axis_scale.to_gnuplot())
        });

    let mut i = 0;

    let max = all_curves
        .iter()
        .map(|&&(_, ref data)| Sample::new(data).mean())
        .fold(::std::f64::NAN, f64::max);

    let mut dummy = [1.0];
    let unit = formatter.scale_values(max, &mut dummy);

    f.configure(Axis::LeftY, |a| {
        a.configure(Grid::Major, |g| g.show())
            .configure(Grid::Minor, |g| g.hide())
            .set(Label(format!("Average time ({})", unit)))
            .set(axis_scale.to_gnuplot())
    });

    // This assumes the curves are sorted. It also assumes that the benchmark IDs all have numeric
    // values or throughputs and that value is sensible (ie. not a mix of bytes and elements
    // or whatnot)
    for (key, group) in &all_curves.iter().group_by(|&&&(ref id, _)| &id.function_id) {
        let mut tuples: Vec<_> = group
            .map(|&&(ref id, ref sample)| {
                // Unwrap is fine here because it will only fail if the assumptions above are not true
                // ie. programmer error.
                let x = id.as_number().unwrap();
                let y = Sample::new(sample).mean();

                (x, y)
            })
            .collect();
        tuples.sort_by(|&(ax, _), &(bx, _)| (ax.partial_cmp(&bx).unwrap_or(Ordering::Less)));
        let (xs, mut ys): (Vec<_>, Vec<_>) = tuples.into_iter().unzip();
        formatter.scale_values(max, &mut ys);

        let function_name = key.as_ref().map(|string| escape_underscores(string));

        f.plot(Lines { x: &xs, y: &ys }, |c| {
            if let Some(name) = function_name {
                c.set(Label(name));
            }
            c.set(LINEWIDTH)
                .set(LineType::Solid)
                .set(COMPARISON_COLORS[i % NUM_COLORS])
        })
        .plot(Points { x: &xs, y: &ys }, |p| {
            p.set(PointType::FilledCircle)
                .set(POINT_SIZE)
                .set(COMPARISON_COLORS[i % NUM_COLORS])
        });

        i += 1;
    }

    debug_script(&path, &f);
    f.set(Output(path)).draw().unwrap()
}

pub fn violin(
    formatter: &dyn ValueFormatter,
    title: &str,
    all_curves: &[&(&BenchmarkId, Vec<f64>)],
    path: &str,
    axis_scale: AxisScale,
) -> Child {
    let path = PathBuf::from(&path);
    let all_curves_vec = all_curves.iter().rev().cloned().collect::<Vec<_>>();
    let all_curves: &[&(&BenchmarkId, Vec<f64>)] = &*all_curves_vec;

    let kdes = all_curves
        .iter()
        .map(|&&(_, ref sample)| {
            let (x, mut y) = kde::sweep(Sample::new(sample), KDE_POINTS, None);
            let y_max = Sample::new(&y).max();
            for y in y.iter_mut() {
                *y /= y_max;
            }

            (x, y)
        })
        .collect::<Vec<_>>();
    let mut xs = kdes
        .iter()
        .flat_map(|&(ref x, _)| x.iter())
        .filter(|&&x| x > 0.);
    let (mut min, mut max) = {
        let &first = xs.next().unwrap();
        (first, first)
    };
    for &e in xs {
        if e < min {
            min = e;
        } else if e > max {
            max = e;
        }
    }
    let mut dummy = [1.0];
    let unit = formatter.scale_values(max, &mut dummy);

    let tics = || (0..).map(|x| (f64::from(x)) + 0.5);
    let size = Size(1280, 200 + (25 * all_curves.len()));
    let mut f = Figure::new();
    f.set(Font(DEFAULT_FONT))
        .set(size)
        .set(Title(format!("{}: Violin plot", escape_underscores(title))))
        .configure(Axis::BottomX, |a| {
            a.configure(Grid::Major, |g| g.show())
                .configure(Grid::Minor, |g| g.hide())
                .set(Label(format!("Average time ({})", unit)))
                .set(axis_scale.to_gnuplot())
        })
        .configure(Axis::LeftY, |a| {
            a.set(Label("Input"))
                .set(Range::Limits(0., all_curves.len() as f64))
                .set(TicLabels {
                    positions: tics(),
                    labels: all_curves
                        .iter()
                        .map(|&&(ref id, _)| escape_underscores(id.as_title())),
                })
        });

    let mut is_first = true;
    for (i, &(ref x, ref y)) in kdes.iter().enumerate() {
        let i = i as f64 + 0.5;
        let mut y1: Vec<_> = y.iter().map(|&y| i + y * 0.5).collect();
        let mut y2: Vec<_> = y.iter().map(|&y| i - y * 0.5).collect();

        formatter.scale_values(max, &mut y1);
        formatter.scale_values(max, &mut y2);

        f.plot(FilledCurve { x: &**x, y1, y2 }, |c| {
            if is_first {
                is_first = false;

                c.set(DARK_BLUE).set(Label("PDF")).set(Opacity(0.25))
            } else {
                c.set(DARK_BLUE).set(Opacity(0.25))
            }
        });
    }
    debug_script(&path, &f);
    f.set(Output(path)).draw().unwrap()
}
