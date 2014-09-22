use std::str::{MaybeOwned, Owned, Slice};

use display::Display;

pub enum Color {
    Black,
    Blue,
    Cyan,
    DarkViolet,
    ForestGreen,
    Gold,
    Gray,
    Green,
    Magenta,
    Red,
    Rgb(u8, u8, u8),
    White,
    Yellow,
}

impl Display<MaybeOwned<'static>> for Color {
    fn display(&self) -> MaybeOwned<'static> {
        match *self {
            Black => Slice("black"),
            Blue => Slice("blue"),
            Cyan => Slice("cyan"),
            DarkViolet => Slice("dark-violet"),
            ForestGreen => Slice("forest-green"),
            Gold => Slice("gold"),
            Gray => Slice("gray"),
            Green => Slice("green"),
            Magenta => Slice("magenta"),
            Red => Slice("red"),
            Rgb(r, g, b) => Owned(format!("#{:02x}{:02x}{:02x}", r, g, b)),
            White => Slice("white"),
            Yellow => Slice("yellow"),
        }
    }
}
