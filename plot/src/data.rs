#![allow(deprecated)]

use std::mem;

use byteorder::{LittleEndian, WriteBytesExt};
use cast::From as _0;

use traits::Data;

macro_rules! impl_data {
    ($($ty:ty),+) => {
        $(
            impl Data for $ty {
                fn f64(self) -> f64 {
                    f64::cast(self)
                }
            }

            impl<'a> Data for &'a $ty {
                fn f64(self) -> f64 {
                    f64::cast(*self)
                }
            }
        )+
    }
}

impl_data!(f32, f64, i16, i32, i64, i8, isize, u16, u32, u64, u8, usize);

#[derive(Clone)]
pub struct Matrix {
    bytes: Vec<u8>,
    ncols: usize,
    nrows: usize,
}

impl Matrix {
    pub fn new<I>(rows: I, scale: <I::Item as Row>::Scale) -> Matrix
    where
        I: Iterator,
        I::Item: Row,
    {
        let ncols = I::Item::ncols();
        let bytes_per_row = ncols * mem::size_of::<f64>();
        let mut bytes = Vec::with_capacity(rows.size_hint().0 * bytes_per_row);

        let mut nrows = 0;
        for row in rows {
            nrows += 1;
            row.append_to(&mut bytes, scale);
        }

        Matrix {
            bytes,
            ncols,
            nrows,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn ncols(&self) -> usize {
        self.ncols
    }

    pub fn nrows(&self) -> usize {
        self.nrows
    }
}

/// Data that can serve as a row of the data matrix
pub trait Row {
    /// Private
    type Scale: Copy;

    /// Append this row to a buffer
    fn append_to(self, buffer: &mut Vec<u8>, scale: Self::Scale);
    /// Number of columns of the row
    fn ncols() -> usize;
}

impl<A, B> Row for (A, B)
where
    A: Data,
    B: Data,
{
    type Scale = (f64, f64);

    fn append_to(self, buffer: &mut Vec<u8>, scale: (f64, f64)) {
        let (a, b) = self;

        buffer.write_f64::<LittleEndian>(a.f64() * scale.0).unwrap();
        buffer.write_f64::<LittleEndian>(b.f64() * scale.1).unwrap();
    }

    fn ncols() -> usize {
        2
    }
}

impl<A, B, C> Row for (A, B, C)
where
    A: Data,
    B: Data,
    C: Data,
{
    type Scale = (f64, f64, f64);

    fn append_to(self, buffer: &mut Vec<u8>, scale: (f64, f64, f64)) {
        let (a, b, c) = self;

        buffer.write_f64::<LittleEndian>(a.f64() * scale.0).unwrap();
        buffer.write_f64::<LittleEndian>(b.f64() * scale.1).unwrap();
        buffer.write_f64::<LittleEndian>(c.f64() * scale.2).unwrap();
    }

    fn ncols() -> usize {
        3
    }
}

impl<A, B, C, D> Row for (A, B, C, D)
where
    A: Data,
    B: Data,
    C: Data,
    D: Data,
{
    type Scale = (f64, f64, f64, f64);

    fn append_to(self, buffer: &mut Vec<u8>, scale: (f64, f64, f64, f64)) {
        let (a, b, c, d) = self;

        buffer.write_f64::<LittleEndian>(a.f64() * scale.0).unwrap();
        buffer.write_f64::<LittleEndian>(b.f64() * scale.1).unwrap();
        buffer.write_f64::<LittleEndian>(c.f64() * scale.2).unwrap();
        buffer.write_f64::<LittleEndian>(d.f64() * scale.3).unwrap();
    }

    fn ncols() -> usize {
        4
    }
}

impl<A, B, C, D, E> Row for (A, B, C, D, E)
where
    A: Data,
    B: Data,
    C: Data,
    D: Data,
    E: Data,
{
    type Scale = (f64, f64, f64, f64, f64);

    #[cfg_attr(feature = "cargo-clippy", allow(many_single_char_names))]
    fn append_to(self, buffer: &mut Vec<u8>, scale: (f64, f64, f64, f64, f64)) {
        let (a, b, c, d, e) = self;

        buffer.write_f64::<LittleEndian>(a.f64() * scale.0).unwrap();
        buffer.write_f64::<LittleEndian>(b.f64() * scale.1).unwrap();
        buffer.write_f64::<LittleEndian>(c.f64() * scale.2).unwrap();
        buffer.write_f64::<LittleEndian>(d.f64() * scale.3).unwrap();
        buffer.write_f64::<LittleEndian>(e.f64() * scale.4).unwrap();
    }

    fn ncols() -> usize {
        5
    }
}
