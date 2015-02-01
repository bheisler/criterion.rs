use std::old_io::MemWriter;
use std::mem;
use std::num::ToPrimitive;

use traits::Data;

macro_rules! impl_data {
    ($($ty:ty),+) => {$(
        impl Data for $ty {
            fn f64(self) -> f64 {
                self.to_f64().unwrap()
            }
        }

        impl<'a> Data for &'a $ty {
            fn f64(self) -> f64 {
                self.to_f64().unwrap()
            }
        }
    )+}
}

impl_data!(f32, f64, i16, i32, i64, i8, isize, u16, u32, u64, u8, usize);

#[derive(Clone)]
pub struct Matrix {
    bytes: Vec<u8>,
    ncols: usize,
    nrows: usize,
}

impl Matrix {
    pub fn new<A, I>(mut rows: I) -> Matrix where A: Row, I: Iterator<Item=A> {
        let ncols = Row::ncols(None::<A>);
        let bytes_per_row = ncols * mem::size_of::<f64>();
        let mut buffer = MemWriter::with_capacity(rows.size_hint().0 * bytes_per_row);

        let mut nrows = 0;
        for row in rows {
            nrows += 1;
            row.append_to(&mut buffer);
        }

        Matrix {
            bytes: buffer.into_inner(),
            ncols: ncols,
            nrows: nrows,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &*self.bytes
    }

    pub fn ncols(&self) -> usize {
        self.ncols
    }

    pub fn nrows(&self) -> usize {
        self.nrows
    }
}

/// Data that can serve as a row of the data matrix
trait Row {
    /// Append this row to a buffer
    fn append_to(self, buffer: &mut MemWriter);
    /// Number of columns of the row
    fn ncols(Option<Self>) -> usize;
}

impl<A, B> Row for (A, B) where A: Data, B: Data {
    fn append_to(self, buffer: &mut MemWriter) {
        let (a, b) = self;

        buffer.write_le_f64(a.f64()).unwrap();
        buffer.write_le_f64(b.f64()).unwrap();
    }

    fn ncols(_: Option<(A, B)>) -> usize {
        2
    }
}

impl<A, B, C> Row for (A, B, C) where A: Data, B: Data, C: Data {
    fn append_to(self, buffer: &mut MemWriter) {
        let (a, b, c) = self;

        buffer.write_le_f64(a.f64()).unwrap();
        buffer.write_le_f64(b.f64()).unwrap();
        buffer.write_le_f64(c.f64()).unwrap();
    }

    fn ncols(_: Option<(A, B, C)>) -> usize {
        3
    }
}

impl<A, B, C, D> Row for (A, B, C, D) where A: Data, B: Data, C: Data, D: Data {
    fn append_to(self, buffer: &mut MemWriter) {
        let (a, b, c, d) = self;

        buffer.write_le_f64(a.f64()).unwrap();
        buffer.write_le_f64(b.f64()).unwrap();
        buffer.write_le_f64(c.f64()).unwrap();
        buffer.write_le_f64(d.f64()).unwrap();
    }

    fn ncols(_: Option<(A, B, C, D)>) -> usize {
        4
    }
}

impl<A, B, C, D, E> Row for (A, B, C, D, E) where A: Data, B: Data, C: Data, D: Data, E: Data {
    fn append_to(self, buffer: &mut MemWriter) {
        let (a, b, c, d, e) = self;

        buffer.write_le_f64(a.f64()).unwrap();
        buffer.write_le_f64(b.f64()).unwrap();
        buffer.write_le_f64(c.f64()).unwrap();
        buffer.write_le_f64(d.f64()).unwrap();
        buffer.write_le_f64(e.f64()).unwrap();
    }

    fn ncols(_: Option<(A, B, C, D, E)>) -> usize {
        5
    }
}
