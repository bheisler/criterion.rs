use std::str::MaybeOwned;

use Script;
use color::Color;
use display::Display;
use {Axes, LineType, PointType};

pub struct Properties {
    axes: Option<Axes>,
    color: Option<Color>,
    label: Option<MaybeOwned<'static>>,
    line_type: Option<LineType>,
    linewidth: Option<f64>,
    point_type: Option<PointType>,
    point_size: Option<f64>,
    style: Style,
}

impl Properties {
    // NB I dislike the visibility rules within the same crate
    #[doc(hidden)]
    pub fn _new(style: Style) -> Properties {
        Properties {
            axes: None,
            color: None,
            label: None,
            line_type: None,
            linewidth: None,
            point_size: None,
            point_type: None,
            style: style,
        }
    }

    /// Select the axes to plot against
    ///
    /// **Note** By default, the `BottomXLeftY` axes are used
    pub fn axes(&mut self, axes: Axes) -> &mut Properties {
        self.axes = Some(axes);
        self
    }

    /// Sets the legend label
    pub fn label<S: IntoMaybeOwned<'static>>(&mut self, label: S) -> &mut Properties {
        self.label = Some(label.into_maybe_owned());
        self
    }

    /// Changes the line type
    pub fn line_type(&mut self, lt: LineType) -> &mut Properties {
        self.line_type = Some(lt);
        self
    }

    /// Changes the width of the line
    ///
    /// # Failure
    ///
    /// Fails if `width` is a non-positive value
    pub fn linewidth(&mut self, width: f64) -> &mut Properties {
        assert!(width > 0.);

        self.linewidth = Some(width);
        self
    }

    /// Changes the size of the points
    ///
    /// # Failure
    ///
    /// Fails if `size` is a non-positive value
    pub fn point_size(&mut self, size: f64) -> &mut Properties {
        assert!(size > 0.);

        self.point_size = Some(size);
        self
    }

    /// Changes the point type
    pub fn point_type(&mut self, pt: PointType) -> &mut Properties {
        self.point_type = Some(pt);
        self
    }

    /// Sets the line color
    pub fn color(&mut self, color: Color) -> &mut Properties {
        self.color = Some(color);
        self
    }
}

#[doc(hidden)]
impl Script for Properties {
    fn script(&self) -> String {
        let mut script = match self.axes {
            None => String::new(),
            Some(axes) => format!("axes {} ", axes.display()),
        };

        script.push_str(format!("with {} ", self.style.display()).as_slice());

        match self.line_type {
            Some(lt) => script.push_str(format!("lt {} ", lt.display()).as_slice()),
            None => {},
        }

        match self.linewidth {
            Some(lw) => script.push_str(format!("lw {} ", lw).as_slice()),
            None => {},
        }

        match self.color {
            Some(color) => {
                script.push_str(format!("lc rgb '{}' ", color.display()).as_slice());
            },
            None => {},
        }

        match self.point_type {
            Some(pt) => script.push_str(format!("pt {} ", pt.display()).as_slice()),
            None => {},
        }

        match self.point_size {
            Some(ps) => script.push_str(format!("ps {} ", ps).as_slice()),
            None => {},
        }

        match self.label {
            Some(ref label) => {
                script.push_str("title '");
                script.push_str(label.as_slice());
                script.push_char('\'')
            },
            None => script.push_str("notitle"),
        }

        script
    }
}

pub enum Style {
    Dots,
    Impulses,
    Lines,
    LinesPoints,
    Points,
    Steps,
}

#[doc(hidden)]
impl Display<&'static str> for Style {
    fn display(&self) -> &'static str {
        match *self {
            Dots => "dots",
            Impulses => "impulses",
            Lines => "lines",
            LinesPoints => "linespoints",
            Points => "points",
            Steps => "steps",
        }
    }
}
