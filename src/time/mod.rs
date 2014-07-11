// XXX I'll love to have a libunits crate, but I'm not sure this is the best approach to static
// unit checking
extern crate time;

use std::fmt::{Formatter,Show};
use std::fmt;
use std::num::One;
use std::num;

use self::traits::{Prefix,Unit};

pub mod prefix;
pub mod traits;
pub mod types;
pub mod unit;

pub fn now() -> types::Ns<u64> {
    Time(time::precise_time_ns())
}

#[deriving(Eq, Ord, PartialEq, PartialOrd)]
pub struct Time<P, U, T>(T);

impl<P, U, T> Time<P, U, T> {
    pub fn unwrap(self) -> T {
        let Time(time) = self;
        time
    }
}

impl<
    P,
    U,
    T: Div<T, T>,
    R: TimeDivRhs<P, U, T, D>,
    D
> Div<R, D>
for Time<P, U, T> {
    fn div(&self, rhs: &R) -> D {
        rhs.reverse_div(self)
    }
}

trait TimeDivRhs<P, U, T, D> {
    fn reverse_div(&self, lhs: &Time<P, U, T>) -> D;
}

impl<
    P,
    U,
    T: Div<T, T>
> TimeDivRhs<P, U, T, T>
for Time<P, U, T> {
    fn reverse_div(&self, lhs: &Time<P, U, T>) -> T {
        let &Time(ref rhs) = self;
        let &Time(ref lhs) = lhs;

        lhs.div(rhs)
    }
}

impl<
    P,
    U
> TimeDivRhs<P, U, f64, Time<P, U, f64>>
for f64 {
    fn reverse_div(&self, lhs: &Time<P, U, f64>) -> Time<P, U, f64> {
        let &Time(ref lhs) = lhs;

        Time(lhs.div(self))
    }
}

impl<
    P,
    U,
    T: Mul<T, T>
> Mul<T, Time<P, U, T>>
for Time<P, U, T> {
    fn mul(&self, rhs: &T) -> Time<P, U, T> {
        let &Time(ref lhs) = self;

        Time(lhs.mul(rhs))
    }
}

impl<
    P: Prefix,
    U: Unit,
    T: Show
> Show
for Time<P, U, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let &Time(ref time) = self;
        let p = Prefix::symbol(None::<P>);
        let u = Unit::symbol(None::<U>);
        try!(time.fmt(f));
        write!(f, " {}{}", p, u)
    }
}

impl<
    P,
    U,
    T: Sub<T, T>
> Sub<Time<P, U, T>, Time<P, U, T>>
for Time<P, U, T> {
    fn sub(&self, rhs: &Time<P, U, T>) -> Time<P, U, T> {
        let &Time(ref lhs) = self;
        let &Time(ref rhs) = rhs;

        Time(lhs.sub(rhs))
    }
}

impl<
    P: Prefix,
    T: Div<T, T> + Mul<T, T> + NumCast + One
> Time<P, unit::Second, T> {
    pub fn to<Q: Prefix>(self) -> Time<Q, unit::Second, T> {
        let p = Prefix::exponent(None::<P>);
        let q = Prefix::exponent(None::<Q>);
        let e = p - q;
        let ten: T = NumCast::from(10u).unwrap();

        if e > 0 {
            Time(self.unwrap() * num::pow(ten, e as uint))
        } else {
            Time(self.unwrap() / num::pow(ten, -e as uint))
        }
    }
}

impl<
    P,
    U,
    T: NumCast
> Time<P, unit::Second, T> {
    pub fn cast<S: NumCast>(self) -> Time<P, U, S> {
        Time(NumCast::from(self.unwrap()).unwrap())
    }
}
