//! Error bar plots

use std::str::SendStr;

use {
    Color, Display, ErrorBarDefault, Figure, Label, LineType, LineWidth, PointSize, PointType,
    Script,
};
use data::Matrix;
use plot::Plot;
use traits::{Data, IntoIterator, Set, mod};

/// Properties common to error bar plots
pub struct Properties {
    color: Option<Color>,
    label: Option<SendStr>,
    line_type: LineType,
    linewidth: Option<f64>,
    point_size: Option<f64>,
    point_type: Option<PointType>,
    style: Style,
}

impl ErrorBarDefault<Style> for Properties {
    fn default(style: Style) -> Properties {
        Properties {
            color: None,
            label: None,
            line_type: LineType::Solid,
            linewidth: None,
            point_type: None,
            point_size: None,
            style: style,
        }
    }
}

impl Script for Properties {
    fn script(&self) -> String {
        let mut script = format!("with {} ", self.style.display());

        script.push_str(format!("lt {} ", self.line_type.display())[]);

        if let Some(lw) = self.linewidth {
            script.push_str(format!("lw {} ", lw)[])
        }

        if let Some(color) = self.color {
            script.push_str(format!("lc rgb '{}' ", color.display())[])
        }

        if let Some(pt) = self.point_type {
            script.push_str(format!("pt {} ", pt.display())[])
        }

        if let Some(ps) = self.point_size {
            script.push_str(format!("ps {} ", ps)[])
        }

        if let Some(ref label) =  self.label {
            script.push_str("title '");
            script.push_str(label.as_slice());
            script.push('\'')
        } else {
            script.push_str("notitle")
        }

        script
    }
}

impl Set<Color> for Properties {
    /// Changes the color of the error bars
    fn set(&mut self, color: Color) -> &mut Properties {

        self.color = Some(color);
        self
    }
}

impl<S> Set<Label<S>> for Properties where S: IntoCow<'static, String, str> {
    /// Sets the legend label
    fn set(&mut self, label: Label<S>) -> &mut Properties {
        self.label = Some(label.0.into_cow());
        self
    }
}

impl Set<LineType> for Properties {
    /// Change the line type
    ///
    /// **Note** By default `Solid` lines are used
    fn set(&mut self, lt: LineType) -> &mut Properties {
        self.line_type = lt;
        self
    }
}

impl Set<LineWidth> for Properties {
    /// Changes the linewidth
    ///
    /// # Panics
    ///
    /// Panics if `lw` is a non-positive value
    fn set(&mut self, lw: LineWidth) -> &mut Properties {
        let lw = lw.0;

        assert!(lw > 0.);

        self.linewidth = Some(lw);
        self
    }
}

impl Set<PointSize> for Properties {
    /// Changes the size of the points
    ///
    /// # Panics
    ///
    /// Panics if `size` is a non-positive value
    fn set(&mut self, ps: PointSize) -> &mut Properties {
        let ps = ps.0;

        assert!(ps > 0.);

        self.point_size = Some(ps);
        self
    }
}

impl Set<PointType> for Properties {
    /// Changes the point type
    fn set(&mut self, pt: PointType) -> &mut Properties {
        self.point_type = Some(pt);
        self
    }
}

#[deriving(Copy)]
enum Style {
    XErrorBars,
    XErrorLines,
    YErrorBars,
    YErrorLines,
}

impl Display<&'static str> for Style {
    fn display(&self) -> &'static str {
        match *self {
            Style::XErrorBars => "xerrorbars",
            Style::XErrorLines => "xerrorlines",
            Style::YErrorBars => "yerrorbars",
            Style::YErrorLines => "yerrorlines",
        }
    }
}

/// Asymmetric error bar plots
pub enum ErrorBar<X, Y, L, H> {
    /// Horizontal error bars
    XErrorBars {
        /// X coordinate of the data points
        x: X,
        /// Y coordinate of the data points
        y: Y,
        /// X coordinate of the left end of the error bar
        x_low: L,
        /// Y coordinate of the right end of the error bar
        x_high: H,
    },
    /// Horizontal error bars, where each point is joined by a line
    XErrorLines {
        /// X coordinate of the data points
        x: X,
        /// Y coordinate of the data points
        y: Y,
        /// X coordinate of the left end of the error bar
        x_low: L,
        /// Y coordinate of the right end of the error bar
        x_high: H,
    },
    /// Vertical error bars
    YErrorBars {
        /// X coordinate of the data points
        x: X,
        /// Y coordinate of the data points
        y: Y,
        /// Y coordinate of the bottom of the error bar
        y_low: L,
        /// Y coordinate of the top of the error bar
        y_high: H,
    },
    /// Vertical error bars, where each point is joined by a line
    YErrorLines {
        /// X coordinate of the data points
        x: X,
        /// Y coordinate of the data points
        y: Y,
        /// Y coordinate of the bottom of the error bar
        y_low: L,
        /// Y coordinate of the top of the error bar
        y_high: H,
    },
}

impl<X, Y, L, H> ErrorBar<X, Y, L, H> {
    fn style(&self) -> Style {
        match *self {
            ErrorBar::XErrorBars { .. } => Style::XErrorBars,
            ErrorBar::XErrorLines { .. } => Style::XErrorLines,
            ErrorBar::YErrorBars { .. } => Style::YErrorBars,
            ErrorBar::YErrorLines { .. } => Style::YErrorLines,
        }
    }
}

impl<A, B, C, D, XI, YI, LI, HI, X, Y, L, H> traits::Plot<ErrorBar<X, Y, L, H>, Properties>
for Figure where
    A: Data, B: Data, C: Data, D: Data,
    XI: Iterator<A>, YI: Iterator<B>, LI: Iterator<C>, HI: Iterator<D>,
    X: IntoIterator<A, XI>, Y: IntoIterator<B, YI>, L: IntoIterator<C, LI>, H: IntoIterator<D, HI>,
{
    fn plot<F>(&mut self, e: ErrorBar<X, Y, L, H>, configure: F) -> &mut Figure where
        F: for<'a> FnOnce(&'a mut Properties) -> &'a mut Properties,
    {
        let style = e.style();
        let (x, y, l, h) = match e {
            ErrorBar::XErrorBars { x, y, x_low, x_high } => (x, y, x_low, x_high),
            ErrorBar::XErrorLines { x, y, x_low, x_high } => (x, y, x_low, x_high),
            ErrorBar::YErrorBars { x, y, y_low, y_high } => (x, y, y_low, y_high),
            ErrorBar::YErrorLines { x, y, y_low, y_high } => (x, y, y_low, y_high),
        };
        let data = Matrix::new(zip!(x, y, l, h));
        self.plots.push(Plot::new(data, configure(&mut ErrorBarDefault::default(style))));
        self
    }
}

// TODO XY error bar
//pub struct XyErrorBar<X, Y, XL, XH, YL, YH> {
    //x: X,
    //y: Y,
    //x_low: XL,
    //x_high: XH,
    //y_low: YL,
    //y_high: YH,
//}

// TODO Symmetric error bars
//pub enum SymmetricErrorBar {
    //XSymmetricErrorBar { x: X, y: Y, x_delta: D },
    //XSymmetricErrorLines { x: X, y: Y, x_delta: D },
    //YSymmetricErrorBar { x: X, y: Y, y_delta: D },
    //YSymmetricErrorLines { x: X, y: Y, y_delta: D },
//}
