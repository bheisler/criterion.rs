use std::mem;

pub struct LinSpace {
    curr: f64,
    state: uint,
    step: f64,
    stop: uint,
}

impl LinSpace {
    pub fn new(start: f64, end: f64, n: uint) -> LinSpace {
        LinSpace {
            curr: start,
            state: 0,
            step: (end - start) / (n - 1) as f64,
            stop: n,
        }
    }
}

impl Iterator<f64> for LinSpace {
    fn next(&mut self) -> Option<f64> {
        let curr = self.curr;

        if self.state < self.stop {
            self.state += 1;

            Some(mem::replace(&mut self.curr, curr + self.step))
        } else {
            None
        }
    }
}
