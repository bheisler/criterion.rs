use super::traits::Prefix;

pub enum No {}
impl Prefix for No {
    fn exponent(_: Option<No>) -> int { 0 }
    fn symbol(_: Option<No>) -> String { "".to_string() }
}

pub enum Mili {}
impl Prefix for Mili {
    fn exponent(_: Option<Mili>) -> int { -3 }
    fn symbol(_: Option<Mili>) -> String { "m".to_string() }
}

pub enum Micro {}
impl Prefix for Micro {
    fn exponent(_: Option<Micro>) -> int { -6 }
    fn symbol(_: Option<Micro>) -> String { "u".to_string() }
}

#[deriving(PartialEq, PartialOrd)]
pub enum Nano {}
impl Prefix for Nano {
    fn exponent(_: Option<Nano>) -> int { -9 }
    fn symbol(_: Option<Nano>) -> String { "n".to_string() }
}
