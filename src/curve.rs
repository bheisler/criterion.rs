use std::str::MaybeOwned;

use Script;
use color::Color;
use display::Display;
use {Axes, LineType, PointType, Solid};

pub struct Properties {
    axes: Option<Axes>,
    color: Option<Color>,
    label: Option<MaybeOwned<'static>>,
    line_type: LineType,
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
            line_type: Solid,
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

    /// Sets the line color
    pub fn color(&mut self, color: Color) -> &mut Properties {
        self.color = Some(color);
        self
    }

    /// Sets the legend label
    pub fn label<S>(&mut self, label: S) -> &mut Properties where S: IntoMaybeOwned<'static> {
        self.label = Some(label.into_maybe_owned());
        self
    }

    /// Changes the line type
    ///
    /// **Note** By default `Solid` lines are used
    pub fn line_type(&mut self, lt: LineType) -> &mut Properties {
        self.line_type = lt;
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
}

#[doc(hidden)]
impl Script for Properties {
    fn script(&self) -> String {
        let mut script = if let Some(axes) = self.axes {
            format!("axes {} ", axes.display())
        } else {
            String::new()
        };

        script.push_str(format!("with {} ", self.style.display()).as_slice());
        script.push_str(format!("lt {} ", self.line_type.display()).as_slice());

        if let Some(lw) = self.linewidth {
            script.push_str(format!("lw {} ", lw).as_slice())
        }

        if let Some(color) = self.color {
            script.push_str(format!("lc rgb '{}' ", color.display()).as_slice())
        }

        if let Some(pt) = self.point_type {
            script.push_str(format!("pt {} ", pt.display()).as_slice())
        }

        if let Some(ps) = self.point_size {
            script.push_str(format!("ps {} ", ps).as_slice())
        }

        if let Some(ref label) = self.label {
            script.push_str("title '");
            script.push_str(label.as_slice());
            script.push('\'')
        } else {
            script.push_str("notitle")
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
