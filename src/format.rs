pub fn change(pct: f64, signed: bool) -> String {
    if signed {
        format!("{:>+6}%", signed_short(pct * 1e2))
    } else {
        format!("{:>6}%", short(pct * 1e2))
    }
}

pub fn time(ns: f64) -> String {
    if ns < 1.0 {
        format!("{:>6} ps", short(ns * 1e3))
    } else if ns < 10f64.powi(3) {
        format!("{:>6} ns", short(ns))
    } else if ns < 10f64.powi(6) {
        format!("{:>6} µs", short(ns / 1e3))
    } else if ns < 10f64.powi(9) {
        format!("{:>6} ms", short(ns / 1e6))
    } else {
        format!("{:>6} s", short(ns / 1e9))
    }
}

pub fn short(n: f64) -> String {
    if n < 10.0 {
        format!("{:.4}", n)
    } else if n < 100.0 {
        format!("{:.3}", n)
    } else if n < 1000.0 {
        format!("{:.2}", n)
    } else if n < 10000.0 {
        format!("{:.1}", n)
    } else {
        format!("{:.0}", n)
    }
}

fn signed_short(n: f64) -> String {
    let n_abs = n.abs();

    let sign = if n >= 0.0 { '+' } else { '\u{2212}' };
    if n_abs < 10.0 {
        format!("{}{:.4}", sign, n_abs)
    } else if n_abs < 100.0 {
        format!("{}{:.3}", sign, n_abs)
    } else if n_abs < 1000.0 {
        format!("{}{:.2}", sign, n_abs)
    } else if n_abs < 10000.0 {
        format!("{}{:.1}", sign, n_abs)
    } else {
        format!("{}{:.0}", sign, n_abs)
    }
}

pub fn iter_count(iterations: u64) -> String {
    if iterations < 10_000 {
        format!("{} iterations", iterations)
    } else if iterations < 1_000_000 {
        format!("{:.0}k iterations", (iterations as f64) / 1000.0)
    } else if iterations < 10_000_000 {
        format!("{:.1}M iterations", (iterations as f64) / (1000.0 * 1000.0))
    } else if iterations < 1_000_000_000 {
        format!("{:.0}M iterations", (iterations as f64) / (1000.0 * 1000.0))
    } else if iterations < 10_000_000_000 {
        format!(
            "{:.1}B iterations",
            (iterations as f64) / (1000.0 * 1000.0 * 1000.0)
        )
    } else {
        format!(
            "{:.0}B iterations",
            (iterations as f64) / (1000.0 * 1000.0 * 1000.0)
        )
    }
}

/// Format a number with thousands separators.
// Based on the corresponding libtest functionality, see
// https://github.com/rust-lang/rust/blob/557359f92512ca88b62a602ebda291f17a953002/library/test/src/bench.rs#L87-L109
fn thousands_sep(mut n: u64, sep: char) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    let mut trailing = false;
    for &pow in &[9, 6, 3, 0] {
        let base = 10_u64.pow(pow);
        if pow == 0 || trailing || n / base != 0 {
            if !trailing {
                write!(output, "{}", n / base).unwrap();
            } else {
                write!(output, "{:03}", n / base).unwrap();
            }
            if pow != 0 {
                output.push(sep);
            }
            trailing = true;
        }
        n %= base;
    }

    output
}

/// Format a value as an integer, including thousands-separators.
pub fn integer(n: f64) -> String {
    thousands_sep(n as u64, ',')
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn short_max_len() {
        let mut float = 1.0;
        while float < 999_999.9 {
            let string = short(float);
            println!("{}", string);
            assert!(string.len() <= 6);
            float *= 2.0;
        }
    }

    #[test]
    fn signed_short_max_len() {
        let mut float = -1.0;
        while float > -999_999.9 {
            let string = signed_short(float);
            println!("{}", string);
            assert!(string.chars().count() <= 7);
            float *= 2.0;
        }
    }

    #[test]
    fn integer_thousands_sep() {
        let n = 140352319.0;
        assert_eq!(integer(n), "140,352,319");
    }
}
