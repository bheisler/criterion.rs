//! "Candlestick" plots

use std::borrow::IntoCow;
use std::string::CowString;

use data::Matrix;
use plot::Plot;
use traits::{Data, IntoIterator, Set, self};
use {Color, Default, Display, Figure, Label, LineType, LineWidth, Script};

/// Properties common to candlestick plots
pub struct Properties {
    color: Option<Color>,
    label: Option<CowString<'static>>,
    line_type: LineType,
    linewidth: Option<f64>,
}

impl Default for Properties {
    fn default() -> Properties {
        Properties {
            color: None,
            label: None,
            line_type: LineType::Solid,
            linewidth: None,
        }
    }
}

impl Script for Properties {
    fn script(&self) -> String {
        let mut script = "with candlesticks ".to_string();

        script.push_str(format!("lt {} ", self.line_type.display())[]);

        if let Some(lw) = self.linewidth {
            script.push_str(format!("lw {} ", lw)[])
        }

        if let Some(color) = self.color {
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

impl Set<Color> for Properties {
    /// Sets the line color
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
    /// Changes the line type
    ///
    /// **Note** By default `Solid` lines are used
    fn set(&mut self, lt: LineType) -> &mut Properties {
        self.line_type = lt;
        self
    }
}

impl Set<LineWidth> for Properties {
    /// Changes the width of the line
    ///
    /// # Panics
    ///
    /// Panics if `width` is a non-positive value
    fn set(&mut self, lw: LineWidth) -> &mut Properties {
        let lw = lw.0;

        assert!(lw > 0.);

        self.linewidth = Some(lw);
        self
    }
}

/// A candlestick consists of a box and two whiskers that extend beyond the box
pub struct Candlesticks<X, WM, BM, BH, WH> {
    /// X coordinate of the candlestick
    pub x: X,
    /// Y coordinate of the end point of the bottom whisker
    pub whisker_min: WM,
    /// Y coordinate of the bottom of the box
    pub box_min: BM,
    /// Y coordinate of the top of the box
    pub box_high: BH,
    /// Y coordinate of the end point of the top whisker
    pub whisker_high: WH,
}

impl<A, B, C, D, E, X, WM, BM, BH, WH, XI, WMI, BMI, BHI, WHI>
traits::Plot<Candlesticks<X, WM, BM, BH, WH>> for Figure where
    A: Data,
    B: Data,
    C: Data,
    D: Data,
    E: Data,
    XI: Iterator<Item=A>,
    WMI: Iterator<Item=B>,
    BMI: Iterator<Item=C>,
    BHI: Iterator<Item=D>,
    WHI: Iterator<Item=E>,
    X: IntoIterator<Iter=XI>,
    WM: IntoIterator<Iter=WMI>,
    BM: IntoIterator<Iter=BMI>,
    BH: IntoIterator<Iter=BHI>,
    WH: IntoIterator<Iter=WHI>,
{
    type Properties = Properties;

    fn plot<F>(
        &mut self,
        candlesticks: Candlesticks<X, WM, BM, BH, WH>,
        configure: F,
    ) -> &mut Figure where
        F: FnOnce(&mut Properties) -> &mut Properties
    {
        let Candlesticks { x, whisker_min, box_min, box_high, whisker_high } = candlesticks;

        let data = Matrix::new(zip!(x, box_min, whisker_min, whisker_high, box_high));
        self.plots.push(Plot::new(data, configure(&mut Default::default())));
        self
    }
}
