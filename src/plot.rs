use data::Matrix;
use Script;

#[deriving(Clone)]
pub struct Plot {
    data: Matrix,
    script: String,
}

impl Plot {
    pub fn new<S: Script>(data: Matrix, script: &S) -> Plot {
        Plot {
            data: data,
            script: script.script(),
        }
    }
    pub fn data(&self) -> &Matrix {
        &self.data
    }

    pub fn script(&self) -> &str {
        self.script[]
    }
}
