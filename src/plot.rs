use data::Matrix;
use Script;

#[derive(Clone)]
pub struct Plot {
    data: Matrix,
    script: String,
}

impl Plot {
    pub fn new<S>(data: Matrix, script: &S) -> Plot where
        S: Script,
    {
        Plot {
            data: data,
            script: script.script(),
        }
    }
    pub fn data(&self) -> &Matrix {
        &self.data
    }

    pub fn script(&self) -> &str {
        &self.script
    }
}
