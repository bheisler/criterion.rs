use super::traits::Unit;

#[deriving(PartialEq, PartialOrd)]
pub enum Second {}
impl Unit for Second { fn symbol(_: Option<Second>) -> String { "s".to_string() } }
