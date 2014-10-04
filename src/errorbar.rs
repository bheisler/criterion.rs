use std::str::MaybeOwned;

use color::Color;
use display::Display;
use {LineType, PointType, Script, Solid};

pub struct Properties {
    color: Option<Color>,
    label: Option<MaybeOwned<'static>>,
    line_type: LineType,
    linewidth: Option<f64>,
    point_size: Option<f64>,
    point_type: Option<PointType>,
    style: Style,
}

impl Properties {
    // NB I dislike the visibility rules within the same crate
    pub fn _new(style: Style) -> Properties {
        Properties {
            color: None,
            label: None,
            line_type: Solid,
            linewidth: None,
            point_type: None,
            point_size: None,
            style: style,
        }
    }

    /// Changes the color of the error bars
    pub fn color(&mut self, color: Color) -> &mut Properties {
        self.color = Some(color);
        self
    }

    /// Sets the legend label
    pub fn label<S>(&mut self, label: S) -> &mut Properties where S: IntoMaybeOwned<'static> {
        self.label = Some(label.into_maybe_owned());
        self
    }

    /// Change the line type
    ///
    /// **Note** By default `Solid` lines are used
    pub fn line_type(&mut self, lt: LineType) -> &mut Properties {
        self.line_type = lt;
        self
    }

    /// Changes the linewidth
    ///
    /// # Failure
    ///
    /// Fails if `lw` is a non-positive value
    pub fn linewidth(&mut self, lw: f64) -> &mut Properties {
        assert!(lw > 0.);

        self.linewidth = Some(lw);
        self
    }

    /// Changes the point type
    pub fn point_type(&mut self, pt: PointType) -> &mut Properties {
        self.point_type = Some(pt);
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
}

#[doc(hidden)]
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

pub enum Style {
    XErrorBar,
    XErrorLines,
    YErrorBar,
    YErrorLines,
}

#[doc(hidden)]
impl Display<&'static str> for Style {
    fn display(&self) -> &'static str {
        match *self {
            XErrorBar => "xerrorbars",
            XErrorLines => "xerrorlines",
            YErrorBar => "yerrorbars",
            YErrorLines => "yerrorlines",
        }
    }
}
