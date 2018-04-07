use format;
use std::io::stdout;
use std::io::Write;
use std::cell::Cell;
use std::fmt;

#[derive(Clone)]
pub struct BenchmarkId {
    pub group_id: String,
    pub function_id: Option<String>,
    pub value_str: Option<String>,
    full_id: String,
}

impl BenchmarkId {
    pub fn new(
        group_id: String,
        function_id: Option<String>,
        value_str: Option<String>,
    ) -> BenchmarkId {
        let full_id = match (&function_id, &value_str) {
            (&Some(ref func), &Some(ref val)) => format!("{}/{}/{}", group_id, func, val),
            (&Some(ref func), &None) => format!("{}/{}", group_id, func),
            (&None, &Some(ref val)) => format!("{}/{}", group_id, val),
            (&None, &None) => group_id.clone(),
        };
        BenchmarkId {
            group_id,
            function_id,
            value_str,
            full_id,
        }
    }

    pub fn id(&self) -> &str {
        &self.full_id
    }
}
impl fmt::Display for BenchmarkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.id())
    }
}

pub(crate) trait Report {
    fn benchmark_start(&self, id: &BenchmarkId);
    fn warmup(&self, id: &BenchmarkId, warmup_ns: f64);
    fn measurement_start(
        &self,
        id: &BenchmarkId,
        sample_count: u64,
        estimate_ns: f64,
        iter_count: u64,
    );
}

pub(crate) struct Reports {
    reports: Vec<Box<Report>>,
}
impl Reports {
    pub fn new(reports: Vec<Box<Report>>) -> Reports {
        Reports { reports }
    }
}
impl Report for Reports {
    fn benchmark_start(&self, id: &BenchmarkId) {
        for report in &self.reports {
            report.benchmark_start(id);
        }
    }

    fn warmup(&self, id: &BenchmarkId, warmup_ns: f64) {
        for report in &self.reports {
            report.warmup(id, warmup_ns);
        }
    }

    fn measurement_start(
        &self,
        id: &BenchmarkId,
        sample_count: u64,
        estimate_ns: f64,
        iter_count: u64,
    ) {
        for report in &self.reports {
            report.measurement_start(id, sample_count, estimate_ns, iter_count);
        }
    }
}

pub(crate) struct CliReport {
    pub enable_text_overwrite: bool,
    pub enable_text_coloring: bool,
    pub verbose: bool,

    last_line_len: Cell<usize>,
}
impl CliReport {
    pub fn new(
        enable_text_overwrite: bool,
        enable_text_coloring: bool,
        verbose: bool,
    ) -> CliReport {
        CliReport {
            enable_text_overwrite: enable_text_overwrite,
            enable_text_coloring: enable_text_coloring,
            verbose: verbose,

            last_line_len: Cell::new(0),
        }
    }

    fn text_overwrite(&self) {
        if self.enable_text_overwrite {
            print!("\r");
            for _ in 0..self.last_line_len.get() {
                print!(" ");
            }
            print!("\r");
        }
    }

    //Passing a String is the common case here.
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    fn print_overwritable(&self, s: String) {
        if self.enable_text_overwrite {
            self.last_line_len.set(s.len());
            print!("{}", s);
            stdout().flush().unwrap();
        } else {
            println!("{}", s);
        }
    }
}
impl Report for CliReport {
    fn benchmark_start(&self, id: &BenchmarkId) {
        self.print_overwritable(format!("Benchmarking {}", id));
    }

    fn warmup(&self, id: &BenchmarkId, warmup_ns: f64) {
        self.text_overwrite();
        self.print_overwritable(format!(
            "Benchmarking {}: Warming up for {}",
            id,
            format::time(warmup_ns)
        ));
    }

    fn measurement_start(
        &self,
        id: &BenchmarkId,
        sample_count: u64,
        estimate_ns: f64,
        iter_count: u64,
    ) {
        self.text_overwrite();
        let iter_string = if self.verbose {
            format!("{} iterations", iter_count)
        } else {
            format::iter_count(iter_count)
        };

        self.print_overwritable(format!(
            "Benchmarking {}: Collecting {} samples in estimated {} ({})",
            id,
            sample_count,
            format::time(estimate_ns),
            iter_string
        ));
    }
}