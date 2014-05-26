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

pub trait AsTime {
    fn as_time(self) -> String;
}

impl AsTime for f64 {
    fn as_time(self) -> String {
        fn short(n: f64) -> String {
            if n < 10.0 { format!("{:.4}", n) }
            else if n < 100.0 { format!("{:.3}", n) }
            else if n < 1000.0 { format!("{:.2}", n) }
            else { format!("{}", n) }
        }

        if self < 0.0 {
            format!("-{}", (-self).as_time())
        } else if self < 1.0 {
            format!("{} ps", short(self * 1e3))
        } else if self < 1_000.0 {
            format!("{} ns", short(self))
        } else if self < 1_000_000.0 {
            format!("{} us", short(self * 1e-3))
        } else if self < 1_000_000_000.0 {
            format!("{} ms", short(self * 1e-6))
        } else {
            format!("{} s", short(self * 1e-9))
        }
    }
}
