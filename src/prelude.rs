//! A collection of the most used traits, structs and enums

pub use {
    Axes, Axis, BoxWidth, Color, Figure, FontSize, Grid, Key, LineType, LineWidth, Opacity, Output,
    PointSize, PointType, Range, Scale, Size, Terminal, TicLabels,
};
pub use candlestick::Candlesticks;
pub use curve::Curve::{Dots, Impulses, Lines, LinesPoints, Points, Steps};
pub use errorbar::ErrorBar::{XErrorBars, XErrorLines, YErrorBars, YErrorLines};
pub use filledcurve::FilledCurve;
pub use key::{Boxed, Horizontal, Justification, Order, Position, Stacked, Vertical};
pub use proxy::{Font, Label, Title};
pub use traits::{Configure, Plot, Set};
