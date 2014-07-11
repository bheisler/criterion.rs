use super::Time;
use super::prefix;
use super::unit;

pub type Ns<T> = Time<prefix::Nano, unit::Second, T>;
