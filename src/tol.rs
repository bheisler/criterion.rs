pub static TOLERANCE: f64 = 1e-5;

pub fn is_close(x: f64, y: f64) -> bool {
    if x == 0. || y == 0. {
        (x - y).abs() < TOLERANCE
    } else {
        (x / y - 1.).abs() < TOLERANCE
    }
}
