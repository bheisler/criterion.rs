//! Simple 2D plotting using `gnuplot`

#![feature(macro_rules)]

use std::collections::TreeMap;
use std::io::{Command, File, IoResult, Process};
use std::num;
use std::str::{MaybeOwned, mod};

use data::Matrix;
use display::Display;
use plot::Plot;

mod data;
mod display;
mod plot;
mod zip;

pub mod axis;
pub mod color;
pub mod curve;
pub mod errorbar;
pub mod filledcurve;
pub mod grid;
pub mod key;

macro_rules! zip {
    ($a:expr, $b:expr) => { $a.zip($b) };
    ($a:expr, $b:expr, $c:expr) => { zip::Zip3::new($a, $b, $c) };
    ($a:expr, $b:expr, $c:expr, $d:expr) => { zip::Zip4::new($a, $b, $c, $d) };
}

/// Plot container
#[deriving(Clone)]
pub struct Figure {
    alpha: Option<f64>,
    axes: TreeMap<axis::Axis, axis::Properties>,
    font: Option<MaybeOwned<'static>>,
    font_size: Option<f64>,
    key: Option<key::Properties>,
    output: Path,
    plots: Vec<Plot>,
    size: Option<(uint, uint)>,
    terminal: Terminal,
    tics: TreeMap<axis::Axis, String>,
    title: Option<MaybeOwned<'static>>,
}

impl Figure {
    /// Creates an empty figure
    pub fn new() -> Figure {
        Figure {
            alpha: None,
            axes: TreeMap::new(),
            font: None,
            font_size: None,
            key: None,
            output: Path::new("output.plot"),
            plots: Vec::new(),
            size: None,
            terminal: Svg,
            tics: TreeMap::new(),
            title: None,
        }
    }

    fn script(&self) -> Vec<u8> {
        let mut s = String::new();

        s.push_str(format!("set output '{}'\n", self.output.display()).as_slice());

        match self.title {
            Some(ref title) => s.push_str(format!("set title '{}'\n", title).as_slice()),
            None => {},
        }

        for axis in self.axes.iter() {
            s.push_str(axis.script().as_slice());
        }

        for (_, script) in self.tics.iter() {
            s.push_str(script.as_slice());
        }

        match self.key {
            Some(ref key) => s.push_str(key.script().as_slice()),
            None => {},
        }

        match self.alpha {
            Some(alpha) => {
                s.push_str(format!("set style fill transparent solid {}\n", alpha).as_slice());
            },
            None => {},
        }

        s.push_str(format!("set terminal {} dashed", self.terminal.display()).as_slice());

        match self.size {
            Some((width, height)) => s.push_str(format!(" size {}, {}", width, height).as_slice()),
            None => {},
        }

        match self.font {
            Some(ref name) => match self.font_size {
                Some(size) => s.push_str(format!(" font '{},{}'", name, size).as_slice()),
                None => s.push_str(format!(" font '{}'", name).as_slice()),
            },
            None => {},
        }

        // TODO This removes the crossbars from the ends of error bars, but should be configurable
        s.push_str("\nunset bars\n");

        let mut is_first_plot = true;
        for plot in self.plots.iter() {
            let data = plot.data();

            if data.bytes().len() == 0 {
                continue;
            }

            if is_first_plot {
                s.push_str("plot ");
                is_first_plot = false;
            } else {
                s.push_str(", ");
            }

            s.push_str(format!(
                    "'-' binary endian=little record={} format='%float64' using ",
                    data.nrows()).as_slice());

            let mut is_first_col = true;
            for col in range(0, data.ncols()) {
                if is_first_col {
                    is_first_col = false;
                } else {
                    s.push_char(':');
                }
                s.push_str((col + 1).to_string().as_slice());
            }
            s.push_char(' ');

            s.push_str(plot.script().as_slice());
        }

        let mut buffer = s.into_bytes();
        let mut is_first = true;
        for plot in self.plots.iter() {
            if is_first {
                is_first = false;
                buffer.push('\n' as u8);
            }
            buffer.push_all(plot.data().bytes());
        }

        buffer
    }

    /// Configures an axis
    ///
    /// # Example
    ///
    /// Based on [`multiaxis.dem`](http://gnuplot.sourceforge.net/demo/multiaxis.html)
    ///
    /// ![Plot](multiaxis.svg)
    ///
    /// ```
    /// # extern crate num;
    /// # extern crate simplot;
    /// # fn main() {
    /// # use std::io::{UserRWX, fs};
    /// use num::Complex;
    /// use simplot::axis::{BottomX, LeftY, Logarithmic, RightY};
    /// use simplot::color::{DarkViolet, Rgb};
    /// use simplot::curve::Lines;
    /// use simplot::grid::Major;
    /// use simplot::key::{Center, Inside, Top};
    /// use simplot::{BottomXRightY, Figure, logspace};
    /// use std::f64::consts::PI;
    ///
    /// fn tf(x: f64) -> Complex<f64> {
    ///     Complex::new(0., x) / Complex::new(10., x) / Complex::new(1., x / 10_000.)
    /// }
    ///
    /// let (start, end) = (1.1, 90_000.);
    /// let xs = logspace(start, end, 101);
    /// let phase = xs.map(|x| tf(x).arg() * 180. / PI);
    /// let magnitude = xs.map(|x| tf(x).norm());
    ///
    /// # fs::mkdir_recursive(&Path::new("target/doc/simplot"), UserRWX).unwrap();
    /// # assert_eq!(Some(String::new()),
    /// Figure::new().
    /// #   font("Helvetica").
    /// #   font_size(12.).
    /// #   output(Path::new("target/doc/simplot/multiaxis.svg")).
    /// #   size((1280, 720)).
    ///     axis(BottomX, |a| a.
    ///         grid(Major, |g| g.
    ///             show()).
    ///         label("Angular frequency (rad/s)").
    ///         range(start, end).
    ///         scale(Logarithmic)).
    ///     axis(LeftY, |a| a.
    ///         label("Gain").
    ///         scale(Logarithmic)).
    ///     axis(RightY, |a| a.
    ///         grid(Major, |g| g.
    ///             show()).
    ///         label("Phase shift (°)")).
    ///     key(|k| k.
    ///         position(Inside(Top, Center)).
    ///         title(" ")).
    ///     title("Frequency response").
    ///     curve(Lines, xs, magnitude, |c| c.
    ///         color(DarkViolet).
    ///         label("Magnitude").
    ///         linewidth(2.)).
    ///     curve(Lines, xs, phase, |c| c.
    ///         axes(BottomXRightY).
    ///         color(Rgb(0, 158, 115)).
    ///         label("Phase").
    ///         linewidth(2.)).
    ///     draw().  // (rest of the chain has been omitted)
    /// #   ok().and_then(|gnuplot| {
    /// #       gnuplot.wait_with_output().ok().and_then(|p| {
    /// #           String::from_utf8(p.error).ok()
    /// #       })
    /// #   }));
    /// # }
    /// ```
    pub fn axis(
        &mut self,
        which: axis::Axis,
        configure: <'a> |&'a mut axis::Properties| -> &'a mut axis::Properties,
    ) -> &mut Figure {
        if self.axes.contains_key(&which) {
            configure(self.axes.find_mut(&which).unwrap());
        } else {
            let mut properties = axis::Properties::_new();
            configure(&mut properties);
            self.axes.insert(which, properties);
        }
        self
    }

    /// Plots a curve
    ///
    /// # Example
    ///
    /// Based on [`simple.dem`](http://gnuplot.sourceforge.net/demo/simple.html)
    ///
    /// ![Plot](curve.svg)
    ///
    /// ```
    /// # use std::io::{UserRWX, fs};
    /// use simplot::color::{DarkViolet, Rgb};
    /// use simplot::curve::{Impulses, LinesPoints, Steps};
    /// use simplot::key::{Inside, Left, Top};
    /// use simplot::{Circle, Dash, Figure, linspace};
    ///
    /// let xs = linspace::<f64>(-10., 10., 51);
    ///
    /// # fs::mkdir_recursive(&Path::new("target/doc/simplot"), UserRWX).unwrap();
    /// # assert_eq!(Some(String::new()),
    /// Figure::new().
    /// #   font("Helvetica").
    /// #   font_size(12.).
    /// #   output(Path::new("target/doc/simplot/curve.svg")).
    /// #   size((1280, 720)).
    ///     key(|k| k.
    ///         boxed().
    ///         position(Inside(Top, Left))).
    ///     curve(LinesPoints, xs, xs.map(|x| x.sin()), |c| c.
    ///         color(DarkViolet).
    ///         label("sin(x)").
    ///         line_type(Dash).
    ///         point_type(Circle).
    ///         point_size(1.5)).
    ///     curve(Steps, xs, xs.map(|x| x.atan()), |c| c.
    ///         color(Rgb(0, 158, 115)).
    ///         label("atan(x)").
    ///         linewidth(2.)).
    ///     curve(Impulses, xs, xs.map(|x| x.atan().cos()), |c| c.
    ///         color(Rgb(86, 180, 233)).
    ///         label("cos(atan(x))")).
    ///     draw().  // (rest of the chain has been omitted)
    /// #   ok().and_then(|gnuplot| {
    /// #       gnuplot.wait_with_output().ok().and_then(|p| {
    /// #           String::from_utf8(p.error).ok()
    /// #       })
    /// #   }));
    /// ```
    pub fn curve<A, B, X, Y>(
        &mut self,
        style: curve::Style,
        x: X,
        y: Y,
        configure: <'a> |&'a mut curve::Properties| -> &'a mut curve::Properties,
    ) -> &mut Figure where
        A: Data, B: Data, X: Iterator<A>, Y: Iterator<B>
    {
        let data = Matrix::new(zip!(x, y));
        self.plots.push(Plot::new(data, configure(&mut curve::Properties::_new(style))));
        self
    }

    /// Spawns a drawing child process
    pub fn draw(&mut self) -> IoResult<Process> {
        let mut gnuplot = try!(Command::new("gnuplot").spawn());
        try!(self.dump(gnuplot.stdin.as_mut().unwrap()));
        Ok(gnuplot)
    }

    /// Dumps the script required to produce the figure into `sink`
    pub fn dump<W: Writer>(&mut self, sink: &mut W) -> IoResult<&mut Figure> {
        try!(sink.write(self.script().as_slice()));
        Ok(self)
    }

    /// Plots error bars
    ///
    /// # Example
    ///
    /// Based on
    /// [Julia plotting tutorial](https://plot.ly/julia/error-bars/#Colored-and-Styled-Error-Bars)
    ///
    /// ![Plot](error_bar.svg)
    ///
    /// ```
    /// # use std::io::{UserRWX, fs};
    /// use simplot::axis::BottomX;
    /// use simplot::color::{DarkViolet, Rgb};
    /// use simplot::curve::Lines;
    /// use simplot::errorbar::YErrorBar;
    /// use simplot::key::{Outside, Right, Top};
    /// use simplot::{Figure, FilledCircle, linspace};
    /// use std::f64::consts::PI;
    /// use std::rand::{Rng, mod};
    ///
    /// fn sinc(mut x: f64) -> f64 {
    ///     if x == 0. {
    ///         1.
    ///     } else {
    ///         x *= PI;
    ///         x.sin() / x
    ///     }
    /// }
    ///
    /// let xs_ = linspace(-4., 4., 101);
    ///
    /// // Fake some data
    /// let mut rng = rand::task_rng();
    /// let xs = linspace(-4_f64, 4., 13).skip(1).take(11).collect::<Vec<f64>>();
    /// let ys = xs.iter().map(|&x| sinc(x) + 0.05 * rng.gen() - 0.025).collect::<Vec<f64>>();
    /// let lows = ys.iter().map(|y| y - 0.025 - 0.075 * rng.gen()).collect::<Vec<f64>>();
    /// let highs = ys.iter().map(|y| y + 0.025 + 0.075 * rng.gen()).collect::<Vec<f64>>();
    /// let xs = xs.iter().map(|x| x + 0.2 * rng.gen() - 0.1);
    ///
    /// # fs::mkdir_recursive(&Path::new("target/doc/simplot"), UserRWX).unwrap();
    /// # assert_eq!(Some(String::new()),
    /// Figure::new().
    /// #   font("Helvetica").
    /// #   font_size(12.).
    /// #   output(Path::new("target/doc/simplot/error_bar.svg")).
    /// #   size((1280, 720)).
    ///     axis(BottomX, |a| a.
    ///         tics([-PI, 0., PI].iter(), ["-π", "0", "π"].iter().map(|&x| x))).
    ///     key(|k| k.
    ///         position(Outside(Top, Right))).
    ///     curve(Lines, xs_, xs_.map(|x| sinc(x)), |c| c.
    ///         color(Rgb(0, 158, 115)).
    ///         label("sinc(x)").
    ///         linewidth(2.)).
    ///     error_bar(YErrorBar, xs, ys.iter(), lows.iter(), highs.iter(), |eb| eb.
    ///         color(DarkViolet).
    ///         linewidth(2.).
    ///         point_type(FilledCircle).
    ///         label("measured")).
    ///     draw().  // (rest of the chain has been omitted)
    /// #   ok().and_then(|gnuplot| {
    /// #       gnuplot.wait_with_output().ok().and_then(|p| {
    /// #           String::from_utf8(p.error).ok()
    /// #       })
    /// #   }));
    /// ```
    pub fn error_bar<A, B, C, D, X, Y, L, H>(
        &mut self,
        style: errorbar::Style,
        x: X,
        y: Y,
        low: L,
        high: H,
        configure: <'a> |&'a mut errorbar::Properties| -> &'a mut errorbar::Properties,
    ) -> &mut Figure where
        A: Data, B: Data, C: Data, D: Data,
        X: Iterator<A>, Y: Iterator<B>, L: Iterator<C>, H: Iterator<D>,
    {
        let data = Matrix::new(zip!(x, y, low, high));
        self.plots.push(Plot::new(data, configure(&mut errorbar::Properties::_new(style))));
        self
    }

    /// Plots a filled curve
    ///
    /// # Example
    ///
    /// Based on [`transparent.dem`](http://gnuplot.sourceforge.net/demo/transparent.html)
    ///
    /// ![Plot](filled_curve.svg)
    ///
    /// ```
    /// # use std::io::{UserRWX, fs};
    /// use simplot::axis::{BottomX, LeftY};
    /// use simplot::color::{ForestGreen, Gold, Red};
    /// use simplot::key::{Inside, Left, LeftJustified, SampleText, Top};
    /// use simplot::{Figure, linspace};
    /// use std::f64::consts::PI;
    /// use std::iter;
    ///
    /// let (start, end) = (-5., 5.);
    /// let xs = linspace(start, end, 101);
    /// let zeros = iter::count(0, 1u).take(1).cycle();
    ///
    /// fn gaussian(x: f64, mu: f64, sigma: f64) -> f64 {
    ///     (((x - mu).powi(2) / 2. / sigma.powi(2)).exp() * sigma * (2. * PI).sqrt()).recip()
    /// }
    ///
    /// # fs::mkdir_recursive(&Path::new("target/doc/simplot"), UserRWX).unwrap();
    /// # assert_eq!(Some(String::new()),
    /// Figure::new().
    /// #   font("Helvetica").
    /// #   font_size(12.).
    /// #   output(Path::new("target/doc/simplot/filled_curve.svg")).
    /// #   size((1280, 720)).
    ///     axis(BottomX, |a| a.
    ///         range(start, end)).
    ///     axis(LeftY, |a| a.
    ///         range(0., 1.)).
    ///     key(|k| k.
    ///         justification(LeftJustified).
    ///         order(SampleText).
    ///         position(Inside(Top, Left)).
    ///         title("Gaussian Distribution")).
    ///     title("Transparent filled curve").
    ///     filled_curve(xs, xs.map(|x| gaussian(x, 0.5, 0.5)), zeros, |fc| fc.
    ///         label("μ = 0.5 σ = 0.5").
    ///         color(ForestGreen)).
    ///     filled_curve(xs, xs.map(|x| gaussian(x, 2.0, 1.0)), zeros, |fc| fc.
    ///         color(Gold).
    ///         label("μ = 2.0 σ = 1.0").
    ///         opacity(0.5)).
    ///     filled_curve(xs, xs.map(|x| gaussian(x, -1.0, 2.0)), zeros, |fc| fc.
    ///         color(Red).
    ///         label("μ = -1.0 σ = 2.0").
    ///         opacity(0.5)).
    ///     draw().  // (rest of the chain has been omitted)
    /// #   ok().and_then(|gnuplot| {
    /// #       gnuplot.wait_with_output().ok().and_then(|p| {
    /// #           String::from_utf8(p.error).ok()
    /// #       })
    /// #   }));
    /// ```
    pub fn filled_curve<A, B, C, X, Y1, Y2>(
        &mut self,
        x: X,
        y1: Y1,
        y2: Y2,
        configure: <'a> |&'a mut filledcurve::Properties| -> &'a mut filledcurve::Properties,
    ) -> &mut Figure where
        A: Data, B: Data, C: Data, X: Iterator<A>, Y1: Iterator<B>, Y2: Iterator<C>
    {
        let data = Matrix::new(zip!(x, y1, y2));
        self.plots.push(Plot::new(data, configure(&mut filledcurve::Properties::_new())));
        self
    }

    /// Changes the font
    pub fn font<S: IntoMaybeOwned<'static>>(&mut self, name: S) -> &mut Figure {
        self.font = Some(name.into_maybe_owned());
        self
    }

    /// Changes the size of the font
    ///
    /// # Failure
    ///
    /// Fails if `size` is a non-positive value
    pub fn font_size(&mut self, size: f64) -> &mut Figure {
        assert!(size >= 0.);

        self.font_size = Some(size);
        self
    }

    /// Configures the key (legend)
    pub fn key(
        &mut self,
        configure: <'a> |&'a mut key::Properties| -> &'a mut key::Properties,
    ) -> &mut Figure {
        if self.key.is_some() {
            configure(self.key.as_mut().unwrap());
        } else {
            let mut key = key::Properties::_new();
            configure(&mut key);
            self.key = Some(key);
        }
        self
    }

    /// Changes the output file
    ///
    /// **Note** The default output file is `output.plot`
    pub fn output(&mut self, path: Path) -> &mut Figure {
        self.output = path;
        self
    }

    /// Saves the script required to produce the figure to `path`
    pub fn save(&mut self, path: &Path) -> IoResult<&mut Figure> {
        try!((try!(File::create(path))).write(self.script().as_slice()))
        Ok(self)
    }

    /// Changes the figure size
    pub fn size(&mut self, (width, height): (uint, uint)) -> &mut Figure {
        self.size = Some((width, height));
        self
    }

    /// Changes the output terminal
    ///
    /// **Note** By default, the terminal is set to `Svg`
    pub fn terminal(&mut self, terminal: Terminal) -> &mut Figure {
        self.terminal = terminal;
        self
    }

    /// Sets the title
    pub fn title<S: IntoMaybeOwned<'static>>(&mut self, title: S) -> &mut Figure {
        self.title = Some(title.into_maybe_owned());
        self
    }
}

/// Iterator that yields equally spaced values in the linear scale
pub struct Linspace<T: Float> {
    start: T,
    step: T,
    state: uint,
    stop: uint,
}

impl<T: Float> DoubleEndedIterator<T> for Linspace<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.state == self.stop {
            None
        } else {
            self.stop -= 1;
            Some(self.start + self.step * num::cast(self.stop).unwrap())
        }
    }
}

impl<T: Float> Iterator<T> for Linspace<T> {
    fn next(&mut self) -> Option<T> {
        if self.state == self.stop {
            None
        } else {
            let next = self.start + self.step * num::cast(self.state).unwrap();
            self.state += 1;
            Some(next)
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let exact = self.stop - self.state;
        (exact, Some(exact))
    }
}

/// Iterator that yields equally spaced values in the logarithmic scale
pub struct Logspace<T: Float> {
    start: T,
    step: T,
    state: uint,
    stop: uint,
}

impl<T: Float> DoubleEndedIterator<T> for Logspace<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.state == self.stop {
            None
        } else {
            self.stop -= 1;
            Some((self.start + self.step * num::cast(self.stop).unwrap()).exp())
        }
    }
}

impl<T: Float> Iterator<T> for Logspace<T> {
    fn next(&mut self) -> Option<T> {
        if self.state == self.stop {
            None
        } else {
            let next = self.start + self.step * num::cast(self.state).unwrap();
            self.state += 1;
            Some(next.exp())
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let exact = self.stop - self.state;
        (exact, Some(exact))
    }
}

/// Pairs of axes that define a coordinate system
pub enum Axes {
    BottomXLeftY,
    BottomXRightY,
    TopXLeftY,
    TopXRightY,
}

pub enum LineType {
    Dash,
    Dot,
    DotDash,
    DotDotDash,
    SmallDot,
    Solid,
}

pub enum PointType {
    Circle,
    FilledCircle,
    FilledSquare,
    FilledTriangle,
    Plus,
    Square,
    Star,
    Triangle,
    X,
}

/// Output terminals
#[deriving(Clone)]
pub enum Terminal {
    Svg,
}

/// Types that can be plotted
pub trait Data {
    fn f64(self) -> f64;
}

// FIXME (rust-lang/rust#16563) Remove `#[doc(hidden)]`
#[doc(hidden)]
trait Script {
    fn script(&self) -> String;
}

pub fn linspace<T: Float>(start: T, end: T, n: uint) -> Linspace<T> {
    let step = if n < 2 {
        // NB The value of `step` doesn't matter in these cases
        num::zero()
    } else {
        (end - start) / num::cast(n - 1).unwrap()
    };

    Linspace {
        start: start,
        state: 0,
        step: step,
        stop: n,
    }
}

pub fn logspace<T: Float>(start: T, end: T, n: uint) -> Logspace<T> {
    assert!(start > num::zero() && end > num::zero());

    let (start, end) = (start.ln(), end.ln());

    let step = if n < 2 {
        // NB The value of `step` doesn't matter in these cases
        num::zero()
    } else {
        (end - start) / num::cast(n - 1).unwrap()
    };

    Logspace {
        start: start,
        state: 0,
        step: step,
        stop: n,
    }
}

/// Returns `gnuplot` version
// FIXME Parsing may fail
pub fn version() -> IoResult<(uint, uint, uint)> {
    let stdout = try!(Command::new("gnuplot").arg("--version").output()).output;
    let mut words = str::from_utf8(stdout.as_slice()).unwrap().words().skip(1);
    let mut version = words.next().unwrap().split('.');
    let major = from_str(version.next().unwrap()).unwrap();
    let minor = from_str(version.next().unwrap()).unwrap();
    let patchlevel = from_str(words.skip(1).next().unwrap()).unwrap();

    Ok((major, minor, patchlevel))
}

#[cfg(test)]
mod test {
    #[test]
    fn version() {
        assert_eq!(super::version().ok().map(|(major, _, _)| major), Some(4));
    }
}
