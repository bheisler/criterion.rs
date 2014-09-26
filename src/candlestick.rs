use std::str::MaybeOwned;

use color::Color;
use display::Display;
use {LineType, Script};

pub struct Properties {
    color: Option<Color>,
    label: Option<MaybeOwned<'static>>,
    line_type: Option<LineType>,
    linewidth: Option<f64>,
}

impl Properties {
    #[doc(hidden)]
    pub fn _new() -> Properties {
        Properties {
            color: None,
            label: None,
            line_type: None,
            linewidth: None,
        }
    }

    /// Sets the line color
    pub fn color(&mut self, color: Color) -> &mut Properties {
        self.color = Some(color);
        self
    }

    /// Sets the legend label
    pub fn label<S: IntoMaybeOwned<'static>>(&mut self, label: S) -> &mut Properties {
        self.label = Some(label.into_maybe_owned());
        self
    }

    /// Changes the line type
    ///
    /// **Note** By default `Solid` lines are used
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

}

impl Script for Properties {
    fn script(&self) -> String {
        let mut script = "with candlesticks ".to_string();

        match self.line_type {
            Some(lt) => script.push_str(format!("lt {} ", lt.display()).as_slice()),
            None => script.push_str("lt -1 "),
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

        match self.label {
            Some(ref label) => {
                script.push_str("title '");
                script.push_str(label.as_slice());
                script.push('\'')
            },
            None => script.push_str("notitle"),
        }

        script
    }
}
