use Throughput;

pub fn change(pct: f64, signed: bool) -> String {
    if signed {
        format!("{:>+6}%", signed_short(pct * 1e2))
    } else {
        format!("{:>6}%", short(pct * 1e2))
    }
}

fn short(n: f64) -> String {
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

    if n_abs < 10.0 {
        format!("{:+.4}", n)
    } else if n_abs < 100.0 {
        format!("{:+.3}", n)
    } else if n_abs < 1000.0 {
        format!("{:+.2}", n)
    } else if n_abs < 10000.0 {
        format!("{:+.1}", n)
    } else {
        format!("{:+.0}", n)
    }
}

pub fn time(ns: f64) -> String {
    if ns < 1.0 {
        format!("{:>6} ps", short(ns * 1e3))
    } else if ns < 10f64.powi(3) {
        format!("{:>6} ns", short(ns))
    } else if ns < 10f64.powi(6) {
        format!("{:>6} us", short(ns / 1e3))
    } else if ns < 10f64.powi(9) {
        format!("{:>6} ms", short(ns / 1e6))
    } else {
        format!("{:>6} s", short(ns / 1e9))
    }
}

pub fn throughput(throughput: &Throughput, ns: f64) -> String {
    match *throughput {
        Throughput::Bytes(bytes) => bytes_per_second(f64::from(bytes) * (1e9 / ns)),
        Throughput::Elements(elems) => elements_per_second(f64::from(elems) * (1e9 / ns)),
    }
}

pub fn bytes_per_second(bytes_per_second: f64) -> String {
    if bytes_per_second < 1024.0 {
        format!("{:>6}   B/s", short(bytes_per_second))
    } else if bytes_per_second < 1024.0 * 1024.0 {
        format!("{:>6} KiB/s", short(bytes_per_second / 1024.0))
    } else if bytes_per_second < 1024.0 * 1024.0 * 1024.0 {
        format!("{:>6} MiB/s", short(bytes_per_second / (1024.0 * 1024.0)))
    } else {
        format!(
            "{:>6} GiB/s",
            short(bytes_per_second / (1024.0 * 1024.0 * 1024.0))
        )
    }
}

pub fn elements_per_second(elements_per_second: f64) -> String {
    if elements_per_second < 1000.0 {
        format!("{:>6}  elem/s", short(elements_per_second))
    } else if elements_per_second < 1000.0 * 1000.0 {
        format!("{:>6} Kelem/s", short(elements_per_second / 1000.0))
    } else if elements_per_second < 1000.0 * 1000.0 * 1000.0 {
        format!(
            "{:>6} Melem/s",
            short(elements_per_second / (1000.0 * 1000.0))
        )
    } else {
        format!(
            "{:>6} Gelem/s",
            short(elements_per_second / (1000.0 * 1000.0 * 1000.0))
        )
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn short_max_len() {
        let mut float = 1.0;
        while float < 999999.9 {
            let string = short(float);
            println!("{}", string);
            assert!(string.len() <= 6);
            float *= 2.0;
        }
    }

    #[test]
    fn signed_short_max_len() {
        let mut float = -1.0;
        while float > -999999.9 {
            let string = signed_short(float);
            println!("{}", string);
            assert!(string.len() <= 7);
            float *= 2.0;
        }
    }
}
