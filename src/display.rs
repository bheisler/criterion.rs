use std::str::SendStr;

use key::{Horizontal, Justification, Order, Stacked, Vertical};
use {Axis, Axes, Color, Display, Grid, LineType, PointType, Terminal};

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

impl Display<SendStr> for Color {
    fn display(&self) -> SendStr {
        match *self {
            Color::Black => "black".into_cow(),
            Color::Blue => "blue".into_cow(),
            Color::Cyan => "cyan".into_cow(),
            Color::DarkViolet => "dark-violet".into_cow(),
            Color::ForestGreen => "forest-green".into_cow(),
            Color::Gold => "gold".into_cow(),
            Color::Gray => "gray".into_cow(),
            Color::Green => "green".into_cow(),
            Color::Magenta => "magenta".into_cow(),
            Color::Red => "red".into_cow(),
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b).into_cow(),
            Color::White => "white".into_cow(),
            Color::Yellow => "yellow".into_cow(),
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

impl Display<&'static str> for Stacked {
    fn display(&self) -> &'static str {
        match *self {
            Stacked::Horizontally => "horizontal",
            Stacked::Vertically => "vertical",
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
