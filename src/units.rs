pub trait ToNanoSeconds {
    fn s(self) -> u64;
    fn ms(self) -> u64;
    fn us(self) -> u64;
    fn ns(self) -> u64;
}

impl ToNanoSeconds for int {
    fn s(self) -> u64 {
        (self as u64) * 1_000_000_000_u64
    }

    fn ms(self) -> u64 {
        (self as u64) * 1_000_000_u64
    }

    fn us(self) -> u64 {
        (self as u64) * 1_000_u64
    }

    fn ns(self) -> u64 {
        self as u64
    }
}

fn short(n: f64) -> String {
    if n < 10.0 { format!("{:.4}", n) }
    else if n < 100.0 { format!("{:.3}", n) }
    else if n < 1000.0 { format!("{:.2}", n) }
    else { format!("{}", n) }
}

fn signed_short(n: f64) -> String {
    let n_abs = n.abs();

    if n_abs < 10.0 { format!("{:+.4}", n) }
    else if n_abs < 100.0 { format!("{:+.3}", n) }
    else if n_abs < 1000.0 { format!("{:+.2}", n) }
    else { format!("{:+}", n) }
}

pub trait AsPercent {
    fn as_percent(&self) -> String;
}

impl AsPercent for f64 {
    fn as_percent(&self) -> String {
        format!("{}%", short(*self * 1e2))
    }
}

pub trait AsSignedPercent {
    fn as_signed_percent(&self) -> String;
}

impl AsSignedPercent for f64 {
    fn as_signed_percent(&self) -> String {
        format!("{}%", signed_short(*self * 1e2))
    }
}

pub trait AsTime {
    fn as_time(&self) -> String;
}

impl AsTime for f64 {
    fn as_time(&self) -> String {
        if *self < 0.0 {
            fail!("negative time");
        } else if *self < 1.0 {
            format!("{} ps", short(*self * 1e3))
        } else if *self < 1_000.0 {
            format!("{} ns", short(*self))
        } else if *self < 1_000_000.0 {
            format!("{} us", short(*self * 1e-3))
        } else if *self < 1_000_000_000.0 {
            format!("{} ms", short(*self * 1e-6))
        } else {
            format!("{} s", short(*self * 1e-9))
        }
    }
}
