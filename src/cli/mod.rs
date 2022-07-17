mod error;
#[cfg(test)]
mod tests;
mod types;

use error::Error;
pub use types::{Color, OutputFormat, PlottingBackend, SaveBaseline};

use std::{env, ffi::OsString};

use crate::benchmark::BenchmarkConfig;

#[derive(Debug, Default, PartialEq)]
pub struct Args {
    pub filter: Option<String>,
    pub color: Color,
    pub verbose: bool,
    pub quiet: bool,
    pub no_plot: bool,
    pub save_baseline: SaveBaseline,
    pub discard_baseline: bool,
    pub baseline: Option<String>,
    pub baseline_lenient: Option<String>,
    pub list: bool,
    pub profile_time: Option<f64>,
    pub export: Option<String>,
    pub compare: bool,
    pub baselines: Vec<String>,
    pub compare_threshold: Option<f64>,
    pub compare_list: bool,
    pub load_baseline: Option<String>,
    pub sample_size: Option<usize>,
    pub warm_up_time: Option<f64>,
    pub measurement_time: Option<f64>,
    pub num_resamples: Option<usize>,
    pub noise_threshold: Option<f64>,
    pub confidence_level: Option<f64>,
    pub significance_level: Option<f64>,
    pub quick: bool,
    pub test: bool,
    pub bench: bool,
    pub plotting_backend: Option<PlottingBackend>,
    pub output_format: OutputFormat,
}

pub fn parse_args(config: &BenchmarkConfig) -> Args {
    let args = env::args_os().collect();
    match try_parse_args(args) {
        Ok(args) => args,
        Err(Error::DisplayHelp) => {
            println!("{}", gen_help(config));
            std::process::exit(0);
        }
        Err(Error::DisplayVersion) => {
            println!("Criterion Benchmark");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error parsing CLI args: {}", e);
            eprintln!("{}", gen_help(config));
            std::process::exit(1);
        }
    }
}

fn try_parse_args(mut args: Vec<OsString>) -> Result<Args, Error> {
    // Remove the executable
    args.remove(0);

    let mut args = pico_args::Arguments::from_vec(args);

    // '--help' and '--version' have higher precedence, so handle them first
    if args.contains(["-h", "--help"]) {
        return Err(Error::DisplayHelp);
    } else if args.contains(["-V", "--version"]) {
        return Err(Error::DisplayVersion);
    }

    // Flags with values first
    let color: Color = args
        .opt_value_from_str("--colour")
        .transpose()
        .or_else(|| args.opt_value_from_str(["-c", "--color"]).transpose())
        .transpose()?
        .unwrap_or_default();
    let save_baseline: SaveBaseline = args
        .opt_value_from_str(["-s", "--save-baseline"])?
        .unwrap_or_default();
    let baseline = args.opt_value_from_str(["-b", "--baseline"])?;
    let baseline_lenient = args.opt_value_from_str("--baseline-lenient")?;
    let profile_time = args.opt_value_from_str("--profile-time")?;
    let export = args.opt_value_from_str("--export")?;
    let baselines: Option<String> = args.opt_value_from_str("--baselines")?;
    let compare_threshold = args.opt_value_from_str("--compare-threshold")?;
    let load_baseline = args.opt_value_from_str("--load-baseline")?;
    let sample_size = args.opt_value_from_str("--sample-size")?;
    let warm_up_time = args.opt_value_from_str("--warm-up-time")?;
    let measurement_time = args.opt_value_from_str("--measurement-time")?;
    let num_resamples = args.opt_value_from_str("--nresamples")?;
    let noise_threshold = args.opt_value_from_str("--noise-threshold")?;
    let confidence_level = args.opt_value_from_str("--confidence-level")?;
    let significance_level = args.opt_value_from_str("--significance-level")?;
    let plotting_backend = args.opt_value_from_str("--plotting-backend")?;
    let output_format: OutputFormat = args
        .opt_value_from_str("--output-format")?
        .unwrap_or_default();

    // Now flags without values
    let verbose = args.contains(["-v", "--verbose"]);
    let quiet = args.contains("--quiet");
    let no_plot = args.contains(["-n", "--noplot"]);
    let discard_baseline = args.contains("--discard-baseline");
    let list = args.contains("--list");
    let compare = args.contains("--compare");
    let compare_list = args.contains("--compare-list");
    let quick = args.contains("--quick");
    let test = args.contains("--test");
    let bench = args.contains("--bench");
    // Ignored values that are parsed for compatibility
    let _ = args.contains("--nocapture");
    let _ = args.contains("--show-output");

    // This needs to be last so that it doesn't get a flag
    let filter = args.opt_free_from_str()?;

    // Finally we fail if there are any remaining args that we didn't handle
    let trailing = args.finish();
    if !trailing.is_empty() {
        return Err(Error::TrailingArgs(trailing));
    }

    // '--baselines' takes a comma separated list as its value
    let baselines: Vec<_> = baselines
        .map(|vals| vals.split(',').map(String::from).collect())
        .unwrap_or_default();

    // Error if flags are missing their requires
    if !baselines.is_empty() && !compare {
        return Err(Error::MissingRequires("--baselines", "--compare"));
    } else if load_baseline.is_some() && baseline.is_none() {
        return Err(Error::MissingRequires("--load-baseline", "--baseline"));
    }

    // Error if there are conflicting args
    if verbose && quiet {
        return Err(Error::ConflictingFlags(&["--verbose", "--quiet"]));
    } else if [
        discard_baseline,
        !save_baseline.is_default(),
        baseline.is_some(),
        baseline_lenient.is_some(),
    ]
    .iter()
    .filter(|b| **b)
    .count()
        > 1
    {
        return Err(Error::ConflictingFlags(&[
            "--discard-baseline",
            "--save-baseline",
            "--baseline",
            "--baseline-lenient",
        ]));
    } else if [
        list,
        test,
        profile_time.is_some(),
        export.is_some(),
        compare,
        load_baseline.is_some(),
    ]
    .iter()
    .filter(|b| **b)
    .count()
        > 1
    {
        return Err(Error::ConflictingFlags(&[
            "--list",
            "--test",
            "--profile-time",
            "--export",
            "--compare",
            "--load-baseline",
        ]));
    } else if quick && sample_size.is_some() {
        return Err(Error::ConflictingFlags(&["--quick", "--sample-size"]));
    }

    Ok(Args {
        baseline,
        baseline_lenient,
        baselines,
        bench,
        color,
        compare,
        compare_list,
        compare_threshold,
        confidence_level,
        discard_baseline,
        export,
        filter,
        list,
        load_baseline,
        measurement_time,
        no_plot,
        noise_threshold,
        num_resamples,
        output_format,
        plotting_backend,
        profile_time,
        quick,
        quiet,
        sample_size,
        save_baseline,
        significance_level,
        test,
        verbose,
        warm_up_time,
    })
}

#[must_use]
fn gen_help(config: &BenchmarkConfig) -> String {
    format!(
        "\
Criterion Benchmark

USAGE:
    <executable> [OPTIONS] [FILTER]

ARGS:
    <FILTER>    Skip benchmarks whose names do not contain FILTER.

OPTIONS:
    -b, --baseline <baseline>
            Compare to a named baseline. If any benchmarks do not have the specified baseline this
            command fails.
        --baseline-lenient <baseline-lenient>
            Compare to a named baseline. If any benchmarks do not have the specified baseline then
            just those benchmarks are not compared against the baseline while every other benchmark
            is compared against the baseline.
        --baselines <baselines>
            Limit the baselines used in tabulated results.
    -c, --color <color>
            Configure coloring of output. always = always colorize output, never = never colorize
            output, auto = colorize output if output is a tty and compiled for unix. [default: auto]
            [possible values: auto, always, never]
        --compare
            Tabulate benchmark results
        --compare-list
            Show benchmark results in a list rather than in a table. Useful when horizontal space is
            limited.
        --compare-threshold <compare-threshold>
            Hide results that differ by less than the threshold percentage. By default, all results
            are shown.
        --confidence-level <confidence-level>
            Changes the default confidence level for this run. [default: {confidence_level}]
        --discard-baseline
            Discard benchmark results.
        --export <export>
            Export baseline as json, printed to stdout
    -h, --help
            Print help information
        --list
            List all benchmarks
        --load-baseline <load-baseline>
            Load a previous baseline instead of sampling new data.
        --measurement-time <measurement-time>
            Changes the default measurement time for this run. [default: {measurement_time}]
    -n, --noplot
            Disable plot and HTML generation.
        --noise-threshold <noise-threshold>
            Changes the default noise threshold for this run. [default: {noise_threshold}]
        --nresamples <nresamples>
            Changes the default number of resamples for this run. [default: {nresamples}]
        --output-format <output-format>
            Change the CLI output format. By default, Criterion.rs will use its own format. If
            output format is set to 'bencher', Criterion.rs will print output in a format that
            resembles the 'bencher' crate. [default: criterion] [possible values: criterion,
            bencher]
        --plotting-backend <plotting-backend>
            Set the plotting backend. By default, Criterion.rs will use the gnuplot backend if
            gnuplot is available, or the plotters backend if it isn't. [possible values: gnuplot,
            plotters]
        --profile-time <profile-time>
            Iterate each benchmark for approximately the given number of seconds, doing no analysis
            and without storing the results. Useful for running the benchmarks in a profiler.
        --quick
            Benchmark only until the significance level has been reached [default: {quick}]
        --quiet
            Print only the benchmark results.
    -s, --save-baseline <save-baseline>
            Save results under a named baseline. [default: base]
        --sample-size <sample-size>
            Changes the default size of the sample for this run. [default: {sample_size}]
        --significance-level <significance-level>
            Changes the default significance level for this run. [default: {significance_level}]
    -v, --verbose
            Print additional statistical information.
        --warm-up-time <warm-up-time>
            Changes the default warm up time for this run. [default: {warm_up_time}]

This executable is a Criterion.rs benchmark.
See https://github.com/bheisler/criterion.rs for more details.

To enable debug output, define the environment variable CRITERION_DEBUG.
Criterion.rs will output more debug information and will save the gnuplot
scripts alongside the generated plots.

To test that the benchmarks work, run `cargo test --benches`

NOTE: If you see an 'unrecognized option' error using any of the options above, see:
https://bheisler.github.io/criterion.rs/book/faq.html
",
        sample_size = config.sample_size,
        warm_up_time = config.warm_up_time.as_secs(),
        measurement_time = config.measurement_time.as_secs(),
        nresamples = config.nresamples,
        noise_threshold = config.noise_threshold,
        confidence_level = config.confidence_level,
        significance_level = config.significance_level,
        quick = config.quick_mode,
    )
}
