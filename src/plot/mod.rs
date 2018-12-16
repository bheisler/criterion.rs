use std::iter;
use std::path::PathBuf;
use std::process::Child;

use criterion_plot::prelude::*;
use stats::univariate::Sample;
use stats::Distribution;

use kde;
use report::BenchmarkId;

pub mod both;
pub mod summary;

mod pdf;
pub(crate) use self::pdf::*;
mod regression;
pub(crate) use self::regression::*;
mod distributions;
pub(crate) use self::distributions::*;

fn escape_underscores(string: &str) -> String {
    string.replace("_", "\\_")
}

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

static DEFAULT_FONT: &'static str = "Helvetica";
static KDE_POINTS: usize = 500;
static SIZE: Size = Size(1280, 720);

const LINEWIDTH: LineWidth = LineWidth(2.);
const POINT_SIZE: PointSize = PointSize(0.75);

const DARK_BLUE: Color = Color::Rgb(31, 120, 180);
const DARK_ORANGE: Color = Color::Rgb(255, 127, 0);
const DARK_RED: Color = Color::Rgb(227, 26, 28);

fn debug_script(path: &PathBuf, figure: &Figure) {
    if ::debug_enabled() {
        let mut script_path = path.clone();
        script_path.set_extension("gnuplot");
        println!("Writing gnuplot script to {:?}", script_path);
        let result = figure.save(script_path.as_path());
        if let Err(e) = result {
            error!("Failed to write debug output: {}", e);
        }
    }
}

pub fn t_test(
    t: f64,
    distribution: &Distribution<f64>,
    id: &BenchmarkId,
    output_directory: &str,
) -> Child {
    let path = PathBuf::from(format!(
        "{}/{}/report/change/t-test.svg",
        output_directory,
        id.as_directory_name()
    ));

    let (xs, ys) = kde::sweep(distribution, KDE_POINTS, None);
    let zero = iter::repeat(0);

    let mut figure = Figure::new();
    figure
        .set(Font(DEFAULT_FONT))
        .set(SIZE)
        .set(Title(format!(
            "{}: Welch t test",
            escape_underscores(id.as_title())
        )))
        .configure(Axis::BottomX, |a| a.set(Label("t score")))
        .configure(Axis::LeftY, |a| a.set(Label("Density")))
        .configure(Key, |k| {
            k.set(Justification::Left)
                .set(Order::SampleText)
                .set(Position::Outside(Vertical::Top, Horizontal::Right))
        })
        .plot(
            FilledCurve {
                x: &*xs,
                y1: &*ys,
                y2: zero,
            },
            |c| {
                c.set(DARK_BLUE)
                    .set(Label("t distribution"))
                    .set(Opacity(0.25))
            },
        )
        .plot(
            Lines {
                x: &[t, t],
                y: &[0, 1],
            },
            |c| {
                c.set(Axes::BottomXRightY)
                    .set(DARK_BLUE)
                    .set(LINEWIDTH)
                    .set(Label("t statistic"))
                    .set(LineType::Solid)
            },
        );
    debug_script(&path, &figure);
    figure.set(Output(path)).draw().unwrap()
}

/// Private
trait Append<T> {
    /// Private
    fn append_(self, item: T) -> Self;
}

// NB I wish this was in the standard library
impl<T> Append<T> for Vec<T> {
    fn append_(mut self, item: T) -> Vec<T> {
        self.push(item);
        self
    }
}
