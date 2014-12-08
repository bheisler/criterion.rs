//! Simple 2D plotting using `gnuplot`
//!
//! # Examples
//!
//! - Simple "curves" (based on [`simple.dem`](http://gnuplot.sourceforge.net/demo/simple.html))
//!
//! ![Plot](curve.svg)
//!
//! ```
//! #![feature(globs)]
//!
//! extern crate simplot;
//! extern crate space;  // https://github.com/japaric/space.rs
//!
//! # use std::io::{USER_RWX, fs};
//! use simplot::prelude::*;
//! use space::linspace;
//! use std::num::FloatMath;
//!
//! # fn main() {
//! let xs = linspace::<f64>(-10., 10., 51);
//!
//! # fs::mkdir_recursive(&Path::new("target/doc/simplot"), USER_RWX).unwrap();
//! # assert_eq!(Some(String::new()),
//! Figure::new().
//! #   set(Font("Helvetica")).
//! #   set(FontSize(12.)).
//! #   set(Output(Path::new("target/doc/simplot/curve.svg"))).
//! #   set(Size(1280, 720)).
//!     configure(Key, |k| k.
//!         set(Boxed::Yes).
//!         set(Position::Inside(Vertical::Top, Horizontal::Left))).
//!     plot(LinesPoints {
//!         x: xs,
//!         y: xs.map(|x| x.sin()),
//!     }, |lp| lp.
//!         set(Color::DarkViolet).
//!         set(Label("sin(x)")).
//!         set(LineType::Dash).
//!         set(PointSize(1.5)).
//!         set(PointType::Circle)).
//!     plot(Steps {
//!         x: xs,
//!         y: xs.map(|x| x.atan()),
//!     }, |s| s.
//!         set(Color::Rgb(0, 158, 115)).
//!         set(Label("atan(x)")).
//!         set(LineWidth(2.))).
//!     plot(Impulses {
//!         x: xs,
//!         y: xs.map(|x| x.atan().cos()),
//!     }, |i| i.
//!         set(Color::Rgb(86, 180, 233)).
//!         set(Label("cos(atan(x))"))).
//!     draw().  // (rest of the chain has been omitted)
//! #   ok().and_then(|gnuplot| {
//! #       gnuplot.wait_with_output().ok().and_then(|p| {
//! #           String::from_utf8(p.error).ok()
//! #       })
//! #   }));
//! # }
//! ```
//!
//! - error bars (based on
//! [Julia plotting tutorial](https://plot.ly/julia/error-bars/#Colored-and-Styled-Error-Bars))
//!
//! ![Plot](error_bar.svg)
//!
//! ```
//! #![feature(globs)]
//!
//! extern crate simplot;
//! extern crate space;  // https://github.com/japaric/space.rs
//!
//! # use std::io::{USER_RWX, fs};
//! use simplot::prelude::*;
//! use space::linspace;
//! use std::f64::consts::PI;
//! use std::num::FloatMath;
//! use std::rand::{Rng, XorShiftRng, mod};
//!
//! fn sinc(mut x: f64) -> f64 {
//!     if x == 0. {
//!         1.
//!     } else {
//!         x *= PI;
//!         x.sin() / x
//!     }
//! }
//!
//! # fn main() {
//! let xs_ = linspace::<f64>(-4., 4., 101);
//!
//! // Fake some data
//! let ref mut rng: XorShiftRng = rand::task_rng().gen();
//! let xs = linspace::<f64>(-4., 4., 13).skip(1).take(11);
//! let ys = xs.map(|x| sinc(x) + 0.05 * rng.gen() - 0.025).collect::<Vec<_>>();
//! let y_low = ys.iter().map(|&y| y - 0.025 - 0.075 * rng.gen()).collect::<Vec<_>>();
//! let y_high = ys.iter().map(|&y| y + 0.025 + 0.075 * rng.gen()).collect::<Vec<_>>();
//! let xs = xs.map(|x| x + 0.2 * rng.gen() - 0.1);
//!
//! # fs::mkdir_recursive(&Path::new("target/doc/simplot"), USER_RWX).unwrap();
//! # assert_eq!(Some(String::new()),
//! Figure::new().
//! #   set(Font("Helvetica")).
//! #   set(FontSize(12.)).
//! #   set(Output(Path::new("target/doc/simplot/error_bar.svg"))).
//! #   set(Size(1280, 720)).
//!     configure(Axis::BottomX, |a| a.
//!         set(TicLabels {
//!             labels: &["-π", "0", "π"],
//!             positions: &[-PI, 0., PI],
//!         })).
//!     configure(Key, |k| k.
//!         set(Position::Outside(Vertical::Top, Horizontal::Right))).
//!     plot(Lines {
//!         x: xs_,
//!         y: xs_.map(|x| sinc(x)),
//!     }, |l| l.
//!         set(Color::Rgb(0, 158, 115)).
//!         set(Label("sinc(x)")).
//!         set(LineWidth(2.))).
//!     plot(YErrorBars {
//!         x: xs,
//!         y: &*ys,
//!         y_low: &*y_low,
//!         y_high: &*y_high,
//!     }, |eb| eb.
//!         set(Color::DarkViolet).
//!         set(LineWidth(2.)).
//!         set(PointType::FilledCircle).
//!         set(Label("measured"))).
//!     draw().  // (rest of the chain has been omitted)
//! #   ok().and_then(|gnuplot| {
//! #       gnuplot.wait_with_output().ok().and_then(|p| {
//! #           String::from_utf8(p.error).ok()
//! #       })
//! #   }));
//! # }
//! ```
//!
//! - Candlesticks (based on
//! [`candlesticks.dem`](http://gnuplot.sourceforge.net/demo/candlesticks.html))
//!
//! ![Plot](candlesticks.svg)
//!
//! ```
//! # #![feature(globs)]
//!
//! extern crate simplot;
//!
//! # use std::io::{USER_RWX, fs};
//! use simplot::prelude::*;
//! use std::rand::{Rng, mod};
//!
//! # fn main() {
//! let xs = range(1u, 11);
//!
//! // Fake some data
//! let mut rng = rand::task_rng();
//! let bh = xs.map(|_| 5f64 + 2.5 * rng.gen()).collect::<Vec<_>>();
//! let bm = xs.map(|_| 2.5f64 + 2.5 * rng.gen()).collect::<Vec<_>>();
//! let wh = bh.iter().map(|&y| y + (10. - y) * rng.gen()).collect::<Vec<_>>();
//! let wm = bm.iter().map(|&y| y * rng.gen()).collect::<Vec<_>>();
//! let m = bm.iter().zip(bh.iter()).map(|(&l, &h)| (h - l) * rng.gen() + l).collect::<Vec<_>>();
//! let m = &*m;
//!
//! # fs::mkdir_recursive(&Path::new("target/doc/simplot"), USER_RWX).unwrap();
//! # assert_eq!(Some(String::new()),
//! Figure::new().
//! #   set(Font("Helvetica")).
//! #   set(FontSize(12.)).
//! #   set(Output(Path::new("target/doc/simplot/candlesticks.svg"))).
//! #   set(Size(1280, 720)).
//!     set(BoxWidth(0.2)).
//!     configure(Axis::BottomX, |a| a.
//!         set(Range::Limits(0., 11.))).
//!     plot(Candlesticks {
//!         x: xs,
//!         whisker_min: &*wm,
//!         box_min: &*bm,
//!         box_high: &*bh,
//!         whisker_high: &*wh,
//!     }, |cs| cs.
//!         set(Color::Rgb(86, 180, 233)).
//!         set(Label("Quartiles")).
//!         set(LineWidth(2.))).
//!     // trick to plot the median
//!     plot(Candlesticks {
//!         x: xs,
//!         whisker_min: m,
//!         box_min: m,
//!         box_high: m,
//!         whisker_high: m,
//!     }, |cs| cs.
//!         set(Color::Black).
//!         set(LineWidth(2.))).
//!     draw().  // (rest of the chain has been omitted)
//! #   ok().and_then(|gnuplot| {
//! #       gnuplot.wait_with_output().ok().and_then(|p| {
//! #           String::from_utf8(p.error).ok()
//! #       })
//! #   }));
//! # }
//! ```
//!
//! - Multiaxis (based on [`multiaxis.dem`](http://gnuplot.sourceforge.net/demo/multiaxis.html))
//!
//! ![Plot](multiaxis.svg)
//!
//! ```
//! # #![feature(globs)]
//!
//! extern crate complex;  // https://github.com/japaric/complex.rs
//! extern crate simplot;
//! extern crate space;  // https://github.com/japaric/space.rs
//!
//! # use std::io::{fs, USER_RWX};
//! use complex::f64::I;
//! use complex::{Complex, Math};
//! use simplot::prelude::*;
//! use space::logspace;
//! use std::f64::consts::PI;
//!
//! fn tf(x: f64) -> Complex<f64> {
//!     (I * x) / (I * x + 10.) / (I * x / 10_000. + 1.)
//! }
//!
//! # fn main() {
//! let (start, end) = (1.1, 90_000.);
//! let xs = logspace(start, end, 101);
//! let phase = xs.map(|x| tf(x).arg() * 180. / PI);
//! let magnitude = xs.map(|x| tf(x).abs());
//!
//! # fs::mkdir_recursive(&Path::new("target/doc/simplot"), USER_RWX).unwrap();
//! # assert_eq!(Some(String::new()),
//! Figure::new().
//! #   set(Font("Helvetica")).
//! #   set(FontSize(12.)).
//! #   set(Output(Path::new("target/doc/simplot/multiaxis.svg"))).
//! #   set(Size(1280, 720)).
//!     set(Title("Frequency response")).
//!     configure(Axis::BottomX, |a| a.
//!         configure(Grid::Major, |g| g.
//!             show()).
//!         set(Label("Angular frequency (rad/s)")).
//!         set(Range::Limits(start, end)).
//!         set(Scale::Logarithmic)).
//!     configure(Axis::LeftY, |a| a.
//!         set(Label("Gain")).
//!         set(Scale::Logarithmic)).
//!     configure(Axis::RightY, |a| a.
//!         configure(Grid::Major, |g| g.
//!             show()).
//!         set(Label("Phase shift (°)"))).
//!     configure(Key, |k| k.
//!         set(Position::Inside(Vertical::Top, Horizontal::Center)).
//!         set(Title(" "))).
//!     plot(Lines {
//!         x: xs,
//!         y: magnitude,
//!     }, |l| l.
//!         set(Color::DarkViolet).
//!         set(Label("Magnitude")).
//!         set(LineWidth(2.))).
//!     plot(Lines {
//!         x: xs,
//!         y: phase,
//!     }, |l| l.
//!         set(Axes::BottomXRightY).
//!         set(Color::Rgb(0, 158, 115)).
//!         set(Label("Phase")).
//!         set(LineWidth(2.))).
//!     draw().  // (rest of the chain has been omitted)
//! #   ok().and_then(|gnuplot| {
//! #       gnuplot.wait_with_output().ok().and_then(|p| {
//! #           String::from_utf8(p.error).ok()
//! #       })
//! #   }));
//! # }
//! ```
//! - Filled curves (based on
//! [`transparent.dem`](http://gnuplot.sourceforge.net/demo/transparent.html))
//!
//! ![Plot](filled_curve.svg)
//!
//! ```
//! # #![feature(globs)]
//!
//! extern crate simplot;
//! extern crate space;  // https://github.com/japaric/space.rs
//!
//! # use std::io::{USER_RWX, fs};
//! use simplot::prelude::*;
//! use space::linspace;
//! use std::f64::consts::PI;
//! use std::iter;
//! use std::num::Float;
//!
//! # fn main() {
//! let (start, end) = (-5., 5.);
//! let xs = linspace(start, end, 101);
//! let zeros = iter::repeat(0u);
//!
//! fn gaussian(x: f64, mu: f64, sigma: f64) -> f64 {
//!     (((x - mu).powi(2) / 2. / sigma.powi(2)).exp() * sigma * (2. * PI).sqrt()).recip()
//! }
//!
//! # fs::mkdir_recursive(&Path::new("target/doc/simplot"), USER_RWX).unwrap();
//! # assert_eq!(Some(String::new()),
//! Figure::new().
//! #   set(Font("Helvetica")).
//! #   set(FontSize(12.)).
//! #   set(Output(Path::new("target/doc/simplot/filled_curve.svg"))).
//! #   set(Size(1280, 720)).
//!     set(Title("Transparent filled curve")).
//!     configure(Axis::BottomX, |a| a.
//!         set(Range::Limits(start, end))).
//!     configure(Axis::LeftY, |a| a.
//!         set(Range::Limits(0., 1.))).
//!     configure(Key, |k| k.
//!         set(Justification::Left).
//!         set(Order::SampleText).
//!         set(Position::Inside(Vertical::Top, Horizontal::Left)).
//!         set(Title("Gaussian Distribution"))).
//!     plot(FilledCurve {
//!         x: xs,
//!         y1: xs.map(|x| gaussian(x, 0.5, 0.5)),
//!         y2: zeros,
//!     }, |fc| fc.
//!         set(Color::ForestGreen).
//!         set(Label("μ = 0.5 σ = 0.5"))).
//!     plot(FilledCurve {
//!         x: xs,
//!         y1: xs.map(|x| gaussian(x, 2.0, 1.0)),
//!         y2: zeros,
//!     }, |fc| fc.
//!         set(Color::Gold).
//!         set(Label("μ = 2.0 σ = 1.0")).
//!         set(Opacity(0.5))).
//!     plot(FilledCurve {
//!         x: xs,
//!         y1: xs.map(|x| gaussian(x, -1.0, 2.0)),
//!         y2: zeros,
//!     }, |fc| fc.
//!         set(Color::Red).
//!         set(Label("μ = -1.0 σ = 2.0")).
//!         set(Opacity(0.5))).
//!     draw().  // (rest of the chain has been omitted)
//! #   ok().and_then(|gnuplot| {
//! #       gnuplot.wait_with_output().ok().and_then(|p| {
//! #           String::from_utf8(p.error).ok()
//! #       })
//! #   }));
//! # }
//! ```

#![deny(missing_docs, warnings)]
#![feature(macro_rules, phase, slicing_syntax, unboxed_closures)]

extern crate zip;
#[phase(plugin)]
extern crate zip_macros;

use std::borrow::Cow::{Borrowed, Owned};
use std::io::{Command, File, IoResult, Process};
use std::str::{SendStr, mod};

use plot::Plot;
use traits::{Configure, Set};

mod data;
mod display;
mod map;
mod plot;

pub mod axis;
pub mod candlestick;
pub mod curve;
pub mod errorbar;
pub mod filledcurve;
pub mod grid;
pub mod key;
pub mod prelude;
pub mod traits;

/// Plot container
pub struct Figure {
    alpha: Option<f64>,
    axes: map::axis::Map<axis::Properties>,
    box_width: Option<f64>,
    font: Option<SendStr>,
    font_size: Option<f64>,
    key: Option<key::Properties>,
    output: Path,
    plots: Vec<Plot>,
    size: Option<(uint, uint)>,
    terminal: Terminal,
    tics: map::axis::Map<String>,
    title: Option<SendStr>,
}

// FIXME (rust-lang/rust#19359) Automatically derive this trait
impl Clone for Figure {
    fn clone(&self) -> Figure {
        Figure {
            alpha: self.alpha,
            axes: self.axes.clone(),
            box_width: self.box_width,
            font: match self.font {
                Some(ref font) => Some(match *font {
                    Borrowed(b) => Borrowed(b),
                    Owned(ref o) => Owned(o.clone()),
                }),
                None => None,
            },
            font_size: self.font_size,
            key: self.key.clone(),
            output: self.output.clone(),
            plots: self.plots.clone(),
            size: self.size,
            terminal: self.terminal,
            tics: self.tics.clone(),
            title: match self.title {
                Some(ref title) => Some(match *title {
                    Borrowed(b) => Borrowed(b),
                    Owned(ref o) => Owned(o.clone()),
                }),
                None => None,
            }
        }
    }
}

impl Figure {
    /// Creates an empty figure
    pub fn new() -> Figure {
        Figure {
            alpha: None,
            axes: map::axis::Map::new(),
            box_width: None,
            font: None,
            font_size: None,
            key: None,
            output: Path::new("output.plot"),
            plots: Vec::new(),
            size: None,
            terminal: Terminal::Svg,
            tics: map::axis::Map::new(),
            title: None,
        }
    }

    fn script(&self) -> Vec<u8> {
        let mut s = String::new();

        s.push_str(format!("set output '{}'\n", self.output.display())[]);

        if let Some(width) = self.box_width {
            s.push_str(&*format!("set boxwidth {}\n", width))
        }

        if let Some(ref title) = self.title {
            s.push_str(&*format!("set title '{}'\n", title))
        }

        for axis in self.axes.iter() {
            s.push_str(&*axis.script());
        }

        for (_, script) in self.tics.iter() {
            s.push_str(&**script);
        }

        if let Some(ref key) = self.key {
            s.push_str(&*key.script())
        }

        if let Some(alpha) = self.alpha {
            s.push_str(&*format!("set style fill transparent solid {}\n", alpha))
        }

        s.push_str(&*format!("set terminal {} dashed", self.terminal.display()));

        if let Some((width, height)) = self.size {
            s.push_str(&*format!(" size {}, {}", width, height))
        }

        if let Some(ref name) = self.font {
            if let Some(size) = self.font_size {
                s.push_str(&*format!(" font '{},{}'", name, size))
            } else {
                s.push_str(&*format!(" font '{}'", name))
            }
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

            s.push_str(&*format!(
                    "'-' binary endian=little record={} format='%float64' using ",
                    data.nrows()));

            let mut is_first_col = true;
            for col in range(0, data.ncols()) {
                if is_first_col {
                    is_first_col = false;
                } else {
                    s.push(':');
                }
                s.push_str((col + 1).to_string()[]);
            }
            s.push(' ');

            s.push_str(plot.script()[]);
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

    /// Spawns a drawing child process
    pub fn draw(&mut self) -> IoResult<Process> {
        let mut gnuplot = try!(Command::new("gnuplot").spawn());
        try!(self.dump(gnuplot.stdin.as_mut().unwrap()));
        Ok(gnuplot)
    }

    /// Dumps the script required to produce the figure into `sink`
    pub fn dump<W>(&mut self, sink: &mut W) -> IoResult<&mut Figure> where W: Writer {
        try!(sink.write(self.script()[]));
        Ok(self)
    }

    /// Saves the script required to produce the figure to `path`
    pub fn save(&mut self, path: &Path) -> IoResult<&mut Figure> {
        try!((try!(File::create(path))).write(self.script()[]))
        Ok(self)
    }
}

impl Configure<Axis, axis::Properties> for Figure {
    /// Configures an axis
    fn configure<F>(&mut self, axis: Axis, configure: F) -> &mut Figure where
        F: for<'a> FnOnce(&'a mut axis::Properties) -> &'a mut axis::Properties,
    {
        if self.axes.contains_key(axis) {
            configure(self.axes.get_mut(axis).unwrap());
        } else {
            let mut properties = Default::default();
            configure(&mut properties);
            self.axes.insert(axis, properties);
        }
        self
    }
}

impl Configure<Key, key::Properties> for Figure {
    /// Configures the key (legend)
    fn configure<F>(&mut self, _: Key, configure: F) -> &mut Figure where
        F: for<'a> FnOnce(&'a mut key::Properties) -> &'a mut key::Properties,
    {
        if self.key.is_some() {
            configure(self.key.as_mut().unwrap());
        } else {
            let mut key = Default::default();
            configure(&mut key);
            self.key = Some(key);
        }
        self
    }
}

impl Set<BoxWidth> for Figure {
    /// Changes the box width of all the box related plots (bars, candlesticks, etc)
    ///
    /// **Note** The default value is 0
    ///
    /// # Panics
    ///
    /// Panics if `width` is a negative value
    fn set(&mut self, width: BoxWidth) -> &mut Figure {
        let width = width.0;

        assert!(width >= 0.);

        self.box_width = Some(width);
        self
    }
}

impl<S> Set<Font<S>> for Figure where S: IntoCow<'static, String, str> {
    /// Changes the font
    fn set(&mut self, font: Font<S>) -> &mut Figure {
        self.font = Some(font.0.into_cow());
        self
    }
}

impl Set<FontSize> for Figure {
    /// Changes the size of the font
    ///
    /// # Panics
    ///
    /// Panics if `size` is a non-positive value
    fn set(&mut self, size: FontSize) -> &mut Figure {
        let size = size.0;

        assert!(size >= 0.);

        self.font_size = Some(size);
        self
    }
}

impl Set<Output> for Figure {
    /// Changes the output file
    ///
    /// **Note** The default output file is `output.plot`
    fn set(&mut self, output: Output) -> &mut Figure {
        self.output = output.0;
        self
    }

}

impl Set<Size> for Figure {
    /// Changes the figure size
    fn set(&mut self, size: Size) -> &mut Figure {
        self.size = Some((size.0, size.1));
        self
    }
}

impl Set<Terminal> for Figure {
    /// Changes the output terminal
    ///
    /// **Note** By default, the terminal is set to `Svg`
    fn set(&mut self, terminal: Terminal) -> &mut Figure {
        self.terminal = terminal;
        self
    }
}

impl<S> Set<Title<S>> for Figure where S: IntoCow<'static, String, str> {
    /// Sets the title
    fn set(&mut self, title: Title<S>) -> &mut Figure {
        self.title = Some(title.0.into_cow());
        self
    }
}

/// Box width for box-related plots: bars, candlesticks, etc
pub struct BoxWidth(pub f64);

/// A font name
pub struct Font<S: IntoCow<'static, String, str>>(pub S);

/// The size of a font
pub struct FontSize(pub f64);

/// The key or legend
pub struct Key;

/// Plot label
pub struct Label<S: IntoCow<'static, String, str>>(pub S);

/// Width of the lines
pub struct LineWidth(pub f64);

/// Fill color opacity
pub struct Opacity(pub f64);

/// Output file path
pub struct Output(pub Path);

/// Size of the points
pub struct PointSize(pub f64);

/// Axis range
pub enum Range {
    /// Autoscale the axis
    Auto,
    /// Set the limits of the axis
    Limits(f64, f64),
}

/// Figure size
pub struct Size(pub uint, pub uint);

/// Labels attached to the tics of an axis
pub struct TicLabels<P, L> {
    /// Labels to attach to the tics
    pub labels: L,
    /// Position of the tics on the axis
    pub positions: P,
}

/// Figure title
pub struct Title<S: IntoCow<'static, String, str>>(pub S);

/// A pair of axes that define a coordinate system
#[allow(missing_docs)]
pub enum Axes {
    BottomXLeftY,
    BottomXRightY,
    TopXLeftY,
    TopXRightY,
}

/// A coordinate axis
#[deriving(FromPrimitive)]
pub enum Axis {
    /// X axis on the bottom side of the figure
    BottomX,
    /// Y axis on the left side of the figure
    LeftY,
    /// Y axis on the right side of the figure
    RightY,
    /// X axis on the top side of the figure
    TopX,
}

/// Color
#[allow(missing_docs)]
pub enum Color {
    Black,
    Blue,
    Cyan,
    DarkViolet,
    ForestGreen,
    Gold,
    Gray,
    Green,
    Magenta,
    Red,
    /// Custom RGB color
    Rgb(u8, u8, u8),
    White,
    Yellow,
}

/// Grid line
#[deriving(FromPrimitive)]
pub enum Grid {
    /// Major gridlines
    Major,
    /// Minor gridlines
    Minor,
}

/// Line type
#[allow(missing_docs)]
pub enum LineType {
    Dash,
    Dot,
    DotDash,
    DotDotDash,
    /// Line made of minimally sized dots
    SmallDot,
    Solid,
}

/// Point type
#[allow(missing_docs)]
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

/// Axis scale
#[allow(missing_docs)]
pub enum Scale {
    Linear,
    Logarithmic,
}

/// Output terminal
#[allow(missing_docs)]
#[deriving(Clone)]
pub enum Terminal {
    Svg,
}

/// Not public version of std::default::Default, used to not leak default constructors into the
/// public API
trait Default {
    /// Creates `Properties` with default configuration
    fn default() -> Self;
}

/// Enums that can produce gnuplot code
trait Display<S> {
    /// Translates the enum in gnuplot code
    fn display(&self) -> S;
}

/// Curve variant of Default
trait CurveDefault<S> {
    /// Creates `curve::Properties` with default configuration
    fn default(S) -> Self;
}

/// Error bar variant of Default
trait ErrorBarDefault<S> {
    /// Creates `errorbar::Properties` with default configuration
    fn default(S) -> Self;
}

/// Structs that can produce gnuplot code
trait Script {
    /// Translates some configuration struct into gnuplot code
    fn script(&self) -> String;
}

/// Returns `gnuplot` version
// FIXME Parsing may fail
pub fn version() -> IoResult<(uint, uint, uint)> {
    let stdout = try!(Command::new("gnuplot").arg("--version").output()).output;
    let mut words = str::from_utf8(stdout[]).unwrap().words().skip(1);
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
