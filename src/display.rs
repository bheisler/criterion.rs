use std::str::MaybeOwned::{Owned, Slice, mod};

use key::{Horizontal, Justification, Order, Stack, Vertical};
use {Axis, Axes, Color, Display, Grid, LineType, PointType, Terminal};
use {curve, errorbar};

impl Display<&'static str> for Axis {
    fn display(&self) -> &'static str {
        match *self {
            Axis::BottomX => "x",
            Axis::LeftY => "y",
            Axis::RightY => "y2",
            Axis::TopX => "x2",
        }
    }
}

impl Display<&'static str> for Axes {
    fn display(&self) -> &'static str {
        match *self {
            Axes::BottomXLeftY => "x1y1",
            Axes::BottomXRightY => "x1y2",
            Axes::TopXLeftY => "x2y1",
            Axes::TopXRightY => "x2y2",
        }
    }
}

impl Display<MaybeOwned<'static>> for Color {
    fn display(&self) -> MaybeOwned<'static> {
        match *self {
            Color::Black => Slice("black"),
            Color::Blue => Slice("blue"),
            Color::Cyan => Slice("cyan"),
            Color::DarkViolet => Slice("dark-violet"),
            Color::ForestGreen => Slice("forest-green"),
            Color::Gold => Slice("gold"),
            Color::Gray => Slice("gray"),
            Color::Green => Slice("green"),
            Color::Magenta => Slice("magenta"),
            Color::Red => Slice("red"),
            Color::Rgb(r, g, b) => Owned(format!("#{:02x}{:02x}{:02x}", r, g, b)),
            Color::White => Slice("white"),
            Color::Yellow => Slice("yellow"),
        }
    }
}

impl Display<&'static str> for Grid {
    fn display(&self) -> &'static str {
        match *self {
            Grid::Major => "",
            Grid::Minor => "m",
        }
    }
}

impl Display<&'static str> for Horizontal {
    fn display(&self) -> &'static str {
        match *self {
            Horizontal::Center => "center",
            Horizontal::Left => "left",
            Horizontal::Right => "right",
        }
    }
}

impl Display<&'static str> for Justification {
    fn display(&self) -> &'static str {
        match *self {
            Justification::Left=> "Left",
            Justification::Right=> "Right",
        }
    }
}

impl Display<&'static str> for LineType {
    fn display(&self) -> &'static str {
        match *self {
            LineType::Dash => "2",
            LineType::Dot => "3",
            LineType::DotDash => "4",
            LineType::DotDotDash => "5",
            LineType::SmallDot => "0",
            LineType::Solid => "1",
        }
    }
}

impl Display<&'static str> for Order {
    fn display(&self) -> &'static str {
        match *self {
            Order::TextSample => "noreverse",
            Order::SampleText => "reverse",
        }
    }
}

impl Display<&'static str> for PointType {
    fn display(&self) -> &'static str {
        match *self {
            PointType::Circle => "6",
            PointType::FilledCircle => "7",
            PointType::FilledSquare => "5",
            PointType::FilledTriangle => "9",
            PointType::Plus => "1",
            PointType::Square => "4",
            PointType::Star => "3",
            PointType::Triangle => "8",
            PointType::X => "2",
        }
    }
}

impl Display<&'static str> for Stack {
    fn display(&self) -> &'static str {
        match *self {
            Stack::Horizontal => "horizontal",
            Stack::Vertical => "vertical",
        }
    }
}

impl Display<&'static str> for Terminal {
    fn display(&self) -> &'static str {
        match *self {
            Terminal::Svg => "svg dynamic",
        }
    }
}

impl Display<&'static str> for Vertical {
    fn display(&self) -> &'static str {
        match *self {
            Vertical::Bottom => "bottom",
            Vertical::Center => "center",
            Vertical::Top => "top",
        }
    }
}

impl Display<&'static str> for curve::Style {
    fn display(&self) -> &'static str {
        match *self {
            curve::Style::Dots => "dots",
            curve::Style::Impulses => "impulses",
            curve::Style::Lines => "lines",
            curve::Style::LinesPoints => "linespoints",
            curve::Style::Points => "points",
            curve::Style::Steps => "steps",
        }
    }
}

impl Display<&'static str> for errorbar::Style {
    fn display(&self) -> &'static str {
        match *self {
            errorbar::Style::XErrorBar => "xerrorbars",
            errorbar::Style::XErrorLines => "xerrorlines",
            errorbar::Style::YErrorBar => "yerrorbars",
            errorbar::Style::YErrorLines => "yerrorlines",
        }
    }
}
