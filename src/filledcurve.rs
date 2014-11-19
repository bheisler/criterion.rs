use std::str::MaybeOwned;

use {Axes, Color, Default, Display, Script};

pub struct Properties {
    axes: Option<Axes>,
    color: Option<Color>,
    label: Option<MaybeOwned<'static>>,
    opacity: Option<f64>,
}

impl Default for Properties {
    fn default() -> Properties {
        Properties {
            axes: None,
            color: None,
            label: None,
            opacity: None,
        }
    }
}

impl Properties {
    /// Select axes to plot against
    ///
    /// **Note** By default, the `BottomXLeftY` axes are used
    pub fn axes(&mut self, axes: Axes) -> &mut Properties {
        self.axes = Some(axes);
        self
    }

    /// Sets the fill color
    pub fn color(&mut self, color: Color) -> &mut Properties {
        self.color = Some(color);
        self
    }

    /// Sets the legend label
    pub fn label<S>(&mut self, label: S) -> &mut Properties where S: IntoMaybeOwned<'static> {
        self.label = Some(label.into_maybe_owned());
        self
    }

    /// Changes the opacity of the fill color
    ///
    /// **Note** By default, the fill color is totally opaque (`opacity = 1.0`)
    ///
    /// # Failure
    ///
    /// Fails if `opacity` is outside the range `[0, 1]`
    pub fn opacity(&mut self, opacity: f64) -> &mut Properties {
        self.opacity = Some(opacity);
        self
    }
}

impl Script for Properties {
    fn script(&self) -> String {
        let mut script = if let Some(axes) = self.axes {
            format!("axes {} ", axes.display())
        } else {
            String::new()
        };
        script.push_str(format!("with filledcurves ")[]);

        script.push_str("fillstyle ");

        if let Some(opacity) = self.opacity {
            script.push_str(format!("solid {} ", opacity)[])
        }

        // TODO border shoulde be configurable
        script.push_str("noborder ");

        if let Some(color) =  self.color {
            script.push_str(format!("lc rgb '{}' ", color.display())[]);
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
