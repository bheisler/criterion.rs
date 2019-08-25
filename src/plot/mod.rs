use std::iter;
use std::path::PathBuf;

use crate::stats::univariate::Sample;
use criterion_plot::prelude::*;

mod distributions;
mod pdf;
mod regression;
mod summary;
mod t_test;
pub(crate) use self::distributions::*;
pub(crate) use self::pdf::*;
pub(crate) use self::regression::*;
pub(crate) use self::summary::*;
pub(crate) use self::t_test::*;

fn escape_underscores(string: &str) -> String {
    string.replace("_", "\\_")
}

static DEFAULT_FONT: &str = "Helvetica";
static KDE_POINTS: usize = 500;
static SIZE: Size = Size(1280, 720);

const LINEWIDTH: LineWidth = LineWidth(2.);
const POINT_SIZE: PointSize = PointSize(0.75);

const DARK_BLUE: Color = Color::Rgb(31, 120, 180);
const DARK_ORANGE: Color = Color::Rgb(255, 127, 0);
const DARK_RED: Color = Color::Rgb(227, 26, 28);

fn debug_script(path: &PathBuf, figure: &Figure) {
    if crate::debug_enabled() {
        let mut script_path = path.clone();
        script_path.set_extension("gnuplot");
        println!("Writing gnuplot script to {:?}", script_path);
        let result = figure.save(script_path.as_path());
        if let Err(e) = result {
            error!("Failed to write debug output: {}", e);
        }
    }
}

/*fn get_max(values: &[f64]) -> f64 {
    assert!(!values.is_empty());
    let mut elems = values.iter();

    match elems.next() {
        Some(&head) => elems.fold(head, |a, &b| a.max(b)),
        // NB `unreachable!` because `Sample` is guaranteed to have at least one data point
        None => unreachable!(),
    }
}*/

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
