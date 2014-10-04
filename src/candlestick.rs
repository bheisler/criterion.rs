use std::str::MaybeOwned;

use color::Color;
use display::Display;
use {LineType, Script, Solid};

pub struct Properties {
    color: Option<Color>,
    label: Option<MaybeOwned<'static>>,
    line_type: LineType,
    linewidth: Option<f64>,
}

impl Properties {
    #[doc(hidden)]
    pub fn _new() -> Properties {
        Properties {
            color: None,
            label: None,
            line_type: Solid,
            linewidth: None,
        }
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

}

impl Script for Properties {
    fn script(&self) -> String {
        let mut script = "with candlesticks ".to_string();

        script.push_str(format!("lt {} ", self.line_type.display()).as_slice());

        if let Some(lw) = self.linewidth {
            script.push_str(format!("lw {} ", lw).as_slice())
        }

        if let Some(color) = self.color {
            script.push_str(format!("lc rgb '{}' ", color.display()).as_slice());
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
