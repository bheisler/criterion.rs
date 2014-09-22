use {Axes, BottomXLeftY, BottomXRightY, TopXLeftY, TopXRightY};
use {LineType, Dash, Dot, DotDash, DotDotDash, SmallDot, Solid};
use {PointType, Circle, FilledCircle, FilledSquare, FilledTriangle, Plus, Square, Star, Triangle,
    X};
use {Terminal, Svg};

pub trait Display<S> {
    fn display(&self) -> S;
}

// NB `Display` is a private trait, but for some reason appears in the documentation
#[doc(hidden)]
impl Display<&'static str> for Axes {
    fn display(&self) -> &'static str {
        match *self {
            BottomXLeftY => "x1y1",
            BottomXRightY => "x1y2",
            TopXLeftY => "x2y1",
            TopXRightY => "x2y2",
        }
    }
}

#[doc(hidden)]
impl Display<&'static str> for LineType {
    fn display(&self) -> &'static str {
        match *self {
            Dash => "2",
            Dot => "3",
            DotDash => "4",
            DotDotDash => "5",
            SmallDot => "0",
            Solid => "1",
        }
    }
}

#[doc(hidden)]
impl Display<&'static str> for PointType {
    fn display(&self) -> &'static str {
        match *self {
            Circle => "6",
            FilledCircle => "7",
            FilledSquare => "5",
            FilledTriangle => "9",
            Plus => "1",
            Square => "4",
            Star => "3",
            Triangle => "8",
            X => "2",
        }
    }
}

#[doc(hidden)]
impl Display<&'static str> for Terminal {
    fn display(&self) -> &'static str {
        match *self {
            Svg => "svg dynamic",
        }
    }
}
