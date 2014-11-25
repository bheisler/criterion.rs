//! Filled curve plots

use std::str::MaybeOwned;

use data::Matrix;
use plot::Plot;
use traits::{Data, IntoIterator, Set, mod};
use {Axes, Color, Default, Display, Figure, Label, Opacity, Script};

/// Properties common to filled curve plots
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

impl Set<Axes> for Properties {
    /// Select axes to plot against
    ///
    /// **Note** By default, the `BottomXLeftY` axes are used
    fn set(&mut self, axes: Axes) -> &mut Properties {
        self.axes = Some(axes);
        self
    }
}

impl Set<Color> for Properties {
    /// Sets the fill color
    fn set(&mut self, color: Color) -> &mut Properties {
        self.color = Some(color);
        self
    }
}

impl<S> Set<Label<S>> for Properties where S: IntoMaybeOwned<'static> {
    /// Sets the legend label
    fn set(&mut self, label: Label<S>) -> &mut Properties {
        self.label = Some(label.0.into_maybe_owned());
        self
    }
}

impl Set<Opacity> for Properties {
    /// Changes the opacity of the fill color
    ///
    /// **Note** By default, the fill color is totally opaque (`opacity = 1.0`)
    ///
    /// # Panics
    ///
    /// Panics if `opacity` is outside the range `[0, 1]`
    fn set(&mut self, opacity: Opacity) -> &mut Properties {
        self.opacity = Some(opacity.0);
        self
    }
}

/// Fills the area between two curves
pub struct FilledCurve<X, Y1, Y2> {
    /// X coordinate of the data points of both curves
    pub x: X,
    /// Y coordinate of the data points of the first curve
    pub y1: Y1,
    /// Y coordinate of the data points of the second curve
    pub y2: Y2,
}

impl<A, B, C, XI, Y1I, Y2I, X, Y1, Y2> traits::Plot<FilledCurve<X, Y1, Y2>, Properties>
for Figure where
    A: Data, B: Data, C: Data,
    XI: Iterator<A>, Y1I: Iterator<B>, Y2I: Iterator<C>,
    X: IntoIterator<A, XI>, Y1: IntoIterator<B, Y1I>, Y2: IntoIterator<C, Y2I>,
{
    fn plot<F>(&mut self, fc: FilledCurve<X, Y1, Y2>, configure: F) -> &mut Figure where
        F: for<'a> FnOnce(&'a mut Properties) -> &'a mut Properties,
    {
        let FilledCurve { x, y1, y2 } = fc;
        let data = Matrix::new(zip!(x, y1, y2));
        self.plots.push(Plot::new(data, configure(&mut Default::default())));
        self
    }
}
