use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use termcolor::{Color, ColorSpec, WriteColor};
use unicode_width::UnicodeWidthStr;

use crate::critcmp::data;
use crate::critcmp::main::Result;

#[derive(Clone, Debug)]
pub struct Comparison {
    name: String,
    benchmarks: Vec<Benchmark>,
    name_to_index: BTreeMap<String, usize>,
}

#[derive(Clone, Debug)]
pub struct Benchmark {
    name: String,
    nanoseconds: f64,
    stddev: Option<f64>,
    throughput: Option<data::Throughput>,
    /// Whether this is the best benchmark in a group. This is only populated
    /// when a `Comparison` is built.
    best: bool,
    /// The rank of this benchmark in a group. The best is always `1.0`. This
    /// is only populated when a `Comparison` is built.
    rank: f64,
}

impl Comparison {
    pub fn new(name: &str, benchmarks: Vec<Benchmark>) -> Comparison {
        let mut comp = Comparison {
            name: name.to_string(),
            benchmarks: benchmarks,
            name_to_index: BTreeMap::new(),
        };
        if comp.benchmarks.is_empty() {
            return comp;
        }

        comp.benchmarks
            .sort_by(|a, b| a.nanoseconds.partial_cmp(&b.nanoseconds).unwrap());
        comp.benchmarks[0].best = true;

        let top = comp.benchmarks[0].nanoseconds;
        for (i, b) in comp.benchmarks.iter_mut().enumerate() {
            comp.name_to_index.insert(b.name.to_string(), i);
            b.rank = b.nanoseconds / top;
        }
        comp
    }

    /// Return the biggest difference, percentage wise, between benchmarks
    /// in this comparison.
    ///
    /// If this comparison has fewer than two benchmarks, then 0 is returned.
    pub fn biggest_difference(&self) -> f64 {
        if self.benchmarks.len() < 2 {
            return 0.0;
        }
        let best = self.benchmarks[0].nanoseconds;
        let worst = self.benchmarks.last().unwrap().nanoseconds;
        ((worst - best) / best) * 100.0
    }

    fn get(&self, name: &str) -> Option<&Benchmark> {
        self.name_to_index
            .get(name)
            .and_then(|&i| self.benchmarks.get(i))
    }
}

impl Benchmark {
    pub fn from_data(b: &data::Benchmark) -> Benchmark {
        Benchmark {
            name: b.fullname().to_string(),
            nanoseconds: b.nanoseconds(),
            stddev: Some(b.stddev()),
            throughput: b.throughput(),
            best: false,
            rank: 0.0,
        }
    }

    pub fn name(self, name: &str) -> Benchmark {
        Benchmark {
            name: name.to_string(),
            ..self
        }
    }
}

pub fn columns<W: WriteColor>(mut wtr: W, groups: &[Comparison]) -> Result<()> {
    let mut columns = BTreeSet::new();
    for group in groups {
        for b in &group.benchmarks {
            columns.insert(b.name.to_string());
        }
    }

    write!(wtr, "group")?;
    for column in &columns {
        write!(wtr, "\t  {}", column)?;
    }
    writeln!(wtr, "")?;

    write_divider(&mut wtr, '-', "group".width())?;
    for column in &columns {
        write!(wtr, "\t  ")?;
        write_divider(&mut wtr, '-', column.width())?;
    }
    writeln!(wtr, "")?;

    for group in groups {
        if group.benchmarks.is_empty() {
            continue;
        }

        write!(wtr, "{}", group.name)?;
        for column_name in &columns {
            let b = match group.get(column_name) {
                Some(b) => b,
                None => {
                    write!(wtr, "\t")?;
                    continue;
                }
            };

            if b.best {
                let mut spec = ColorSpec::new();
                spec.set_fg(Some(Color::Green)).set_bold(true);
                wtr.set_color(&spec)?;
            }
            write!(
                wtr,
                "\t  {:<5.2} {:>14} {:>14}",
                b.rank,
                time(b.nanoseconds, b.stddev),
                throughput(b.throughput),
            )?;
            if b.best {
                wtr.reset()?;
            }
        }
        writeln!(wtr, "")?;
    }
    Ok(())
}

pub fn rows<W: WriteColor>(mut wtr: W, groups: &[Comparison]) -> Result<()> {
    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            writeln!(wtr, "")?;
        }
        rows_one(&mut wtr, group)?;
    }
    Ok(())
}

fn rows_one<W: WriteColor>(mut wtr: W, group: &Comparison) -> Result<()> {
    writeln!(wtr, "{}", group.name)?;
    write_divider(&mut wtr, '-', group.name.width())?;
    writeln!(wtr, "")?;

    if group.benchmarks.is_empty() {
        writeln!(wtr, "NOTHING TO SHOW")?;
        return Ok(());
    }

    for b in &group.benchmarks {
        writeln!(
            wtr,
            "{}\t{:>7.2}\t{:>15}\t{:>12}",
            b.name,
            b.rank,
            time(b.nanoseconds, b.stddev),
            throughput(b.throughput),
        )?;
    }
    Ok(())
}

fn write_divider<W: WriteColor>(mut wtr: W, divider: char, width: usize) -> Result<()> {
    let div: String = iter::repeat(divider).take(width).collect();
    write!(wtr, "{}", div)?;
    Ok(())
}

fn time(nanos: f64, stddev: Option<f64>) -> String {
    const MIN_MICRO: f64 = 2_000.0;
    const MIN_MILLI: f64 = 2_000_000.0;
    const MIN_SEC: f64 = 2_000_000_000.0;

    let (div, label) = if nanos < MIN_MICRO {
        (1.0, "ns")
    } else if nanos < MIN_MILLI {
        (1_000.0, "µs")
    } else if nanos < MIN_SEC {
        (1_000_000.0, "ms")
    } else {
        (1_000_000_000.0, "s")
    };
    if let Some(stddev) = stddev {
        format!("{:.1}±{:.2}{}", nanos / div, stddev / div, label)
    } else {
        format!("{:.1}{}", nanos / div, label)
    }
}

fn throughput(throughput: Option<data::Throughput>) -> String {
    use data::Throughput::*;
    match throughput {
        Some(Bytes(num)) => throughput_per(num, "B"),
        Some(Elements(num)) => throughput_per(num, "Elem"),
        _ => "? ?/sec".to_string(),
    }
}

fn throughput_per(per: f64, unit: &str) -> String {
    const MIN_K: f64 = (2 * (1 << 10) as u64) as f64;
    const MIN_M: f64 = (2 * (1 << 20) as u64) as f64;
    const MIN_G: f64 = (2 * (1 << 30) as u64) as f64;

    if per < MIN_K {
        format!("{} {}/sec", per as u64, unit)
    } else if per < MIN_M {
        format!("{:.1} K{}/sec", (per / (1 << 10) as f64), unit)
    } else if per < MIN_G {
        format!("{:.1} M{}/sec", (per / (1 << 20) as f64), unit)
    } else {
        format!("{:.1} G{}/sec", (per / (1 << 30) as f64), unit)
    }
}
