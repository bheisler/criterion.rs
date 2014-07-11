use super::Time;
use super::prefix;
use super::unit;

pub trait Unit { fn symbol(_: Option<Self>) -> String;}

pub trait Prefix {
    fn exponent(_: Option<Self>) -> int;
    fn symbol(_: Option<Self>) -> String;
}

#[experimental]
pub trait Second { fn s(self) -> Time<prefix::No, unit::Second, Self> { Time(self) } }
impl<T> Second for T {}

#[experimental]
pub trait Milisecond { fn ms(self) -> Time<prefix::Mili, unit::Second, Self> { Time(self) } }
impl<T> Milisecond for T {}

pub trait Microsecond { fn us(self) -> Time<prefix::Micro, unit::Second, Self> { Time(self) } }
impl<T> Microsecond for T {}

pub trait Nanosecond { fn ns(self) -> Time<prefix::Nano, unit::Second, Self> { Time(self) } }
impl<T> Nanosecond for T {}
