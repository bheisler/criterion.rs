//! A collection of the most used traits, structs and enums

pub use candlestick::Candlesticks;
pub use curve::Curve::{Dots, Impulses, Lines, LinesPoints, Points, Steps};
pub use errorbar::ErrorBar::{XErrorBars, XErrorLines, YErrorBars, YErrorLines};
pub use filledcurve::FilledCurve;
pub use key::{Boxed, Horizontal, Justification, Order, Position, Stacked, Vertical};
pub use proxy::{Font, Label, Output, Title};
pub use traits::{Configure, Plot, Set};
pub use {Axes, Axis, BoxWidth, Color, Figure, FontSize, Grid, Key, LineType, LineWidth, Opacity,
         PointSize, PointType, Range, Scale, ScaleFactor, Size, Terminal, TicLabels};
