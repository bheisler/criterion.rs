#[macro_use]
extern crate criterion;
extern crate serde_json;
extern crate tempdir;
extern crate walkdir;

use criterion::{Benchmark, Criterion, Fun, ParameterizedBenchmark, Throughput};
use serde_json::value::Value;
use std::cell::RefCell;
use std::cmp::max;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::time::Duration;
use std::time::SystemTime;
use tempdir::TempDir;
use walkdir::WalkDir;

/*
 * Please note that these tests are not complete examples of how to use
 * Criterion.rs. See the benches folder for actual examples.
 */
fn temp_dir() -> TempDir {
    TempDir::new("").unwrap()
}

// Configure a Criterion struct to perform really fast benchmarks. This is not
// recommended for real benchmarking, only for testing.
fn short_benchmark(dir: &TempDir) -> Criterion {
    Criterion::default()
        .output_directory(dir.path())
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_millis(500))
        .nresamples(1000)
}

#[derive(Clone)]
struct Counter {
    counter: Rc<RefCell<usize>>,
}
impl Counter {
    fn count(&self) {
        *(*self.counter).borrow_mut() += 1;
    }

    fn read(&self) -> usize {
        *(*self.counter).borrow()
    }
}
impl Default for Counter {
    fn default() -> Counter {
        Counter {
            counter: Rc::new(RefCell::new(0)),
        }
    }
}

fn create_command(depth: usize) -> Command {
    let mut command = Command::new("python3");
    command
        .arg("tests/external_process.py")
        .arg(format!("{}", depth));
    command
}

fn create_command_without_arg() -> Command {
    let mut command = Command::new("python3");
    command.arg("tests/external_process.py");
    command
}

fn has_python3() -> bool {
    Command::new("python3")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .is_ok()
}

#[test]
fn test_creates_directory() {
    let dir = temp_dir();
    short_benchmark(&dir).bench_function("test_creates_directory", |b| b.iter(|| 10));
    assert!(dir.path().join("test_creates_directory").is_dir());
}

#[test]
fn test_without_plots() {
    let dir = temp_dir();
    short_benchmark(&dir)
        .without_plots()
        .bench_function("test_without_plots", |b| b.iter(|| 10));

    for entry in WalkDir::new(dir.path().join("test_without_plots")) {
        let entry = entry.ok();
        let is_svg = entry
            .as_ref()
            .and_then(|entry| entry.path().extension())
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "svg")
            .unwrap_or(false);
        assert!(
            !is_svg,
            "Found SVG file ({:?}) in output directory with plots disabled",
            entry.unwrap().file_name()
        );
    }
}

#[test]
fn test_without_overwrite() {
    // Seed the directory with previous results
    let dir = temp_dir();
    short_benchmark(&dir).bench_function("test_without_overwrite", |b| b.iter(|| 10));

    let mut newest_update: Option<SystemTime> = None;

    for entry in WalkDir::new(dir.path().join("test_without_overwrite")) {
        let entry = entry.ok().unwrap();
        let modified = entry.metadata().unwrap().modified().unwrap();
        newest_update = match newest_update {
            Some(latest) => Some(max(latest, modified)),
            None => Some(modified),
        };
    }

    short_benchmark(&dir)
        .without_overwrite()
        .bench_function("test_without_overwrite", |b| b.iter(|| 10));

    let mut found_modified = false;
    for entry in WalkDir::new(dir.path().join("test_without_overwrite")) {
        let entry = entry.ok().unwrap();
        let path = entry.path().to_owned();
        let modified = entry.metadata().unwrap().modified().unwrap();
        found_modified = (modified == newest_update.unwrap()) || found_modified;
        if modified > newest_update.unwrap() {
            panic!(format!(
                "found modified file with disabled overwrite: {:?}",
                path
            ))
        }
    }
    assert!(
        found_modified,
        "failed to find file with known modification time"
    );
}

#[test]
fn test_sample_size() {
    let dir = temp_dir();
    let counter = Counter::default();

    let clone = counter.clone();
    short_benchmark(&dir)
        .sample_size(50)
        .bench_function("test_sample_size", move |b| {
            clone.count();
            b.iter(|| 10)
        });

    // This function will be called more than sample_size times because of the
    // warmup.
    assert!(counter.read() > 50);
}

#[test]
fn test_warmup_time() {
    let dir = temp_dir();
    let counter1 = Counter::default();

    let clone = counter1.clone();
    short_benchmark(&dir)
        .warm_up_time(Duration::from_millis(100))
        .bench_function("test_warmup_time_1", move |b| {
            clone.count();
            b.iter(|| 10)
        });

    let counter2 = Counter::default();
    let clone = counter2.clone();
    short_benchmark(&dir)
        .warm_up_time(Duration::from_millis(2000))
        .bench_function("test_warmup_time_2", move |b| {
            clone.count();
            b.iter(|| 10)
        });

    assert!(counter1.read() < counter2.read());
}

#[test]
fn test_measurement_time() {
    let dir = temp_dir();
    let counter1 = Counter::default();

    let clone = counter1.clone();
    short_benchmark(&dir)
        .measurement_time(Duration::from_millis(100))
        .bench_function("test_meas_time_1", move |b| b.iter(|| clone.count()));

    let counter2 = Counter::default();
    let clone = counter2.clone();
    short_benchmark(&dir)
        .measurement_time(Duration::from_millis(2000))
        .bench_function("test_meas_time_2", move |b| b.iter(|| clone.count()));

    assert!(counter1.read() < counter2.read());
}

#[test]
fn test_bench_function() {
    let dir = temp_dir();
    short_benchmark(&dir).bench_function("test_bench_function", move |b| b.iter(|| 10));
}

#[test]
fn test_bench_functions() {
    let dir = temp_dir();
    let function_1 = Fun::new("times 10", |b, i| b.iter(|| *i * 10));
    let function_2 = Fun::new("times 20", |b, i| b.iter(|| *i * 20));

    let functions = vec![function_1, function_2];

    short_benchmark(&dir).bench_functions("test_bench_functions", functions, 20);
}

#[test]
fn test_bench_function_over_inputs() {
    let dir = temp_dir();
    short_benchmark(&dir).bench_function_over_inputs(
        "test_bench_function_over_inputs",
        |b, i| b.iter(|| *i * 10),
        vec![100, 1000],
    );
}

#[test]
fn test_bench_program() {
    if !has_python3() {
        return;
    }

    let dir = temp_dir();
    short_benchmark(&dir).bench_program("test_bench_program", create_command(10));
}

#[test]
fn test_bench_program_over_inputs() {
    if !has_python3() {
        return;
    }

    let dir = temp_dir();

    // Note that bench_program_over_inputs automatically passes the input
    // as the first command-line parameter.
    short_benchmark(&dir).bench_program_over_inputs(
        "test_bench_program_over_inputs",
        create_command_without_arg,
        vec![10, 20],
    );
}

#[test]
fn test_bench_unparameterized() {
    let dir = temp_dir();
    let mut benchmark = Benchmark::new("return 10", |b| b.iter(|| 10))
        .with_function("return 20", |b| b.iter(|| 20));

    if has_python3() {
        benchmark = benchmark.with_program("external", create_command(10));
    }

    short_benchmark(&dir).bench("test_bench_unparam", benchmark);
}

#[test]
fn test_bench_parameterized() {
    let dir = temp_dir();
    let parameters = vec![5, 10];
    let mut benchmark =
        ParameterizedBenchmark::new("times 10", |b, i| b.iter(|| *i * 10), parameters)
            .with_function("times 20", |b, i| b.iter(|| *i * 20));

    if has_python3() {
        // Unlike bench_program_over_inputs, the parameter is provided as a
        // parameter to the closure here.
        benchmark = benchmark.with_program("external", |i| create_command(*i));
    }

    short_benchmark(&dir).bench("test_bench_param", benchmark);
}

#[test]
fn test_filtering() {
    let dir = temp_dir();
    let counter = Counter::default();
    let clone = counter.clone();

    short_benchmark(&dir)
        .with_filter("Foo")
        .bench_function("test_filtering", move |b| b.iter(|| clone.count()));

    assert_eq!(counter.read(), 0);
    assert!(!dir.path().join("test_filtering").is_dir());
}

#[test]
fn test_timing_loops() {
    let dir = temp_dir();
    short_benchmark(&dir).bench(
        "test_timing_loops",
        Benchmark::new("iter", |b| b.iter(|| 10))
            .with_function("iter_with_setup", |b| {
                b.iter_with_setup(|| vec![10], |v| v[0])
            })
            .with_function("iter_with_large_setup", |b| {
                b.iter_with_large_setup(|| vec![10], ::std::mem::drop)
            })
            .with_function("iter_with_large_drop", |b| {
                b.iter_with_large_drop(|| vec![10; 100])
            }),
    );
}

#[test]
fn test_throughput() {
    let dir = temp_dir();
    short_benchmark(&dir).bench(
        "test_throughput_bytes",
        Benchmark::new("strlen", |b| b.iter(|| "foo".len())).throughput(Throughput::Bytes(3)),
    );
    short_benchmark(&dir).bench(
        "test_throughput_elems",
        ParameterizedBenchmark::new(
            "veclen",
            |b, v| b.iter(|| v.len()),
            vec![vec![1], vec![1, 2, 3]],
        ).throughput(|v| Throughput::Elements(v.len() as u32)),
    );
}

// Verify that all expected output files are present
#[test]
fn test_output_files() {
    let tempdir = temp_dir();
    for _ in 0..2 {
        short_benchmark(&tempdir).bench(
            "test_output",
            Benchmark::new("output_1", |b| b.iter(|| 10))
                .with_function("output_2", |b| b.iter(|| 20)),
        );
    }

    fn verify_file(dir: &PathBuf, path: &str) -> PathBuf {
        let full_path = dir.join(path);
        assert!(
            full_path.is_file(),
            "File {:?} does not exist or is not a file",
            full_path
        );
        let metadata = full_path.metadata().unwrap();
        assert!(metadata.len() > 0);
        full_path
    }

    fn verify_json(dir: &PathBuf, path: &str) {
        let full_path = verify_file(dir, path);
        let f = File::open(full_path).unwrap();
        serde_json::from_reader::<File, Value>(f).unwrap();
    }

    fn verify_svg(dir: &PathBuf, path: &str) {
        verify_file(dir, path);
    }

    fn verify_html(dir: &PathBuf, path: &str) {
        verify_file(dir, path);
    }

    for x in 0..2 {
        let dir = tempdir.path().join(format!("test_output/output_{}", x + 1));

        verify_json(&dir, "new/estimates.json");
        verify_json(&dir, "new/sample.json");
        verify_json(&dir, "new/tukey.json");
        verify_json(&dir, "change/estimates.json");

        if short_benchmark(&tempdir).can_plot() && cfg!(feature = "html_reports") {
            verify_svg(&dir, "report/MAD.svg");
            verify_svg(&dir, "report/mean.svg");
            verify_svg(&dir, "report/median.svg");
            verify_svg(&dir, "report/pdf.svg");
            verify_svg(&dir, "report/regression.svg");
            verify_svg(&dir, "report/SD.svg");
            verify_svg(&dir, "report/slope.svg");
            verify_svg(&dir, "report/both/pdf.svg");
            verify_svg(&dir, "report/both/regression.svg");
            verify_svg(&dir, "report/change/mean.svg");
            verify_svg(&dir, "report/change/median.svg");
            verify_svg(&dir, "report/change/t-test.svg");

            verify_svg(&dir, "report/pdf_small.svg");
            verify_svg(&dir, "report/regression_small.svg");
            verify_svg(&dir, "report/relative_pdf_small.svg");
            verify_svg(&dir, "report/relative_regression_small.svg");
            verify_html(&dir, "report/index.html");
        }
    }

    if short_benchmark(&tempdir).can_plot() && cfg!(feature = "html_reports") {
        let dir = tempdir.path().join("test_output");

        verify_svg(&dir, "report/violin.svg");
        verify_html(&dir, "report/index.html");
    }

    short_benchmark(&tempdir).final_summary();

    if short_benchmark(&tempdir).can_plot() && cfg!(feature = "html_reports") {
        let dir = tempdir.path().to_owned();

        verify_html(&dir, "report/index.html");
    }
}

#[test]
#[should_panic(expected = "Benchmark function must call Bencher::iter or related method.")]
fn test_bench_with_no_iteration_panics() {
    let dir = temp_dir();
    short_benchmark(&dir).bench("test_no_iter", Benchmark::new("no_iter", |_b| {}));
}

mod macros {
    use super::criterion;

    #[test]
    #[should_panic(expected = "group executed")]
    fn criterion_main() {
        fn group() {}
        fn group2() {
            panic!("group executed");
        }

        criterion_main!(group, group2);

        main();
    }

    #[test]
    fn criterion_main_trailing_comma() {
        // make this a compile-only check
        // as the second logger initialization causes panic
        #[allow(dead_code)]
        fn group() {}
        #[allow(dead_code)]
        fn group2() {}

        criterion_main!(group, group2,);

        // silence dead_code warning
        if false {
            main()
        }
    }

    #[test]
    #[should_panic(expected = "group executed")]
    fn criterion_group() {
        use self::criterion::Criterion;

        fn group(_crit: &mut Criterion) {}
        fn group2(_crit: &mut Criterion) {
            panic!("group executed");
        }

        criterion_group!(test_group, group, group2);

        test_group();
    }

    #[test]
    #[should_panic(expected = "group executed")]
    fn criterion_group_trailing_comma() {
        use self::criterion::Criterion;

        fn group(_crit: &mut Criterion) {}
        fn group2(_crit: &mut Criterion) {
            panic!("group executed");
        }

        criterion_group!(test_group, group, group2,);

        test_group();
    }

}
