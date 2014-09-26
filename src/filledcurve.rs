use std::str::MaybeOwned;

use Axes;
use Script;
use color::Color;
use display::Display;

pub struct Properties {
    axes: Option<Axes>,
    color: Option<Color>,
    label: Option<MaybeOwned<'static>>,
    opacity: Option<f64>,
}

impl Properties {
    // NB I dislike the visibility rules within the same crate
    #[doc(hidden)]
    pub fn _new() -> Properties {
        Properties {
            axes: None,
            color: None,
            label: None,
            opacity: None,
        }
    }

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
    pub fn label<S: IntoMaybeOwned<'static>>(&mut self, label: S) -> &mut Properties {
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

#[doc(hidden)]
impl Script for Properties {
    fn script(&self) -> String {
        let mut script = match self.axes {
            None => String::new(),
            Some(axes) => format!("axes {} ", axes.display()),
        };
        script.push_str(format!("with filledcurves ").as_slice());

        script.push_str("fillstyle ");

        match self.opacity {
            Some(opacity) => {
                script.push_str(format!("solid {} ", opacity).as_slice())
            },
            None => {},
        }

        // TODO border shoulde be configurable
        script.push_str("noborder ");

        match self.color {
            Some(ref color) => {
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
