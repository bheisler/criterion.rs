mod error;
#[cfg(test)]
mod tests;
mod types;

use error::Error;
pub use types::{Color, OutputFormat, PlottingBackend, SaveBaseline};

use std::{env, ffi::OsString, str::FromStr};

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
            eprintln!("Use '--help' for more details on usage");
            std::process::exit(1);
        }
    }
}

fn parse_value<T: FromStr>(flag: &'static str, parser: &mut lexopt::Parser) -> error::Result<T> {
    match parser.value() {
        Ok(arg) => match arg.clone().into_string() {
            Ok(s) => s.parse().map_err(|_| Error::InvalidFlagValue(flag, arg)),
            Err(_) => Err(Error::InvalidFlagValue(flag, arg)),
        },
        Err(_) => Err(Error::FlagMissingValue(flag)),
    }
}

fn try_parse_args(raw_args: Vec<OsString>) -> Result<Args, Error> {
    use lexopt::prelude::*;

    let mut args = Args::default();
    let mut parser = lexopt::Parser::from_iter(raw_args.into_iter());
    while let Some(arg) = parser.next()? {
        match arg {
            // '--help' and '--version' bail parsing immediately
            Short('h') | Long("help") => return Err(Error::DisplayHelp),
            Short('V') | Long("version") => return Err(Error::DisplayVersion),
            // All the flags with values
            Short('c') | Long("color") | Long("colour") => {
                args.color = parse_value("--color", &mut parser)?;
            }
            Short('s') | Long("save-baseline") => {
                args.save_baseline = parse_value("--save-baseline", &mut parser)?;
            }
            Short('b') | Long("baseline") => {
                args.baseline = Some(parse_value("--baseline", &mut parser)?);
            }
            Long("baseline-lenient") => {
                args.baseline_lenient = Some(parse_value("--baseline-lenient", &mut parser)?);
            }
            Long("profile-time") => {
                args.profile_time = Some(parse_value("--profile-time", &mut parser)?);
            }
            Long("export") => {
                args.export = Some(parse_value("--export", &mut parser)?);
            }
            Long("baselines") => {
                let baselines: String = parse_value("--baselines", &mut parser)?;
                args.baselines = baselines.split(',').map(String::from).collect();
            }
            Long("compare-threshold") => {
                args.compare_threshold = Some(parse_value("--compare-threshold", &mut parser)?);
            }
            Long("load-baseline") => {
                args.load_baseline = Some(parse_value("--load-baseline", &mut parser)?);
            }
            Long("sample-size") => {
                args.sample_size = Some(parse_value("--sample-size", &mut parser)?);
            }
            Long("warm-up-time") => {
                args.warm_up_time = Some(parse_value("--warm-up-time", &mut parser)?);
            }
            Long("measurement-time") => {
                args.measurement_time = Some(parse_value("--measurement-time", &mut parser)?);
            }
            Long("nresamples") => {
                args.num_resamples = Some(parse_value("--nresamples", &mut parser)?);
            }
            Long("noise-threshold") => {
                args.noise_threshold = Some(parse_value("--noise-threshold", &mut parser)?);
            }
            Long("confidence-level") => {
                args.confidence_level = Some(parse_value("--confidence-level", &mut parser)?);
            }
            Long("significance-level") => {
                args.significance_level = Some(parse_value("--significance-level", &mut parser)?);
            }
            Long("plotting-backend") => {
                args.plotting_backend = Some(parse_value("--plotting-backend", &mut parser)?);
            }
            Long("output-format") => {
                args.output_format = parse_value("--output-format", &mut parser)?;
            }
            // All the flags without values
            Short('v') | Long("verbose") => args.verbose = true,
            Long("quiet") => args.quiet = true,
            Short('n') | Long("noplot") => args.no_plot = true,
            Long("discard-baseline") => args.discard_baseline = true,
            Long("list") => args.list = true,
            Long("compare") => args.compare = true,
            Long("compare-list") => args.compare_list = true,
            Long("quick") => args.quick = true,
            Long("test") => args.test = true,
            Long("bench") => args.bench = true,
            // '--nocapture' and '--show-output' are kept for libtest compatibility
            Long("nocapture") => {}
            Long("show-output") => {}
            // Values
            Value(val) if args.filter.is_none() => {
                args.filter = Some(val.parse()?);
            }
            _ => return Err(Error::UnexpectedArg(format!("{:?}", arg))),
        }
    }

    // Error if flags are missing their requires
    if !args.baselines.is_empty() && !args.compare {
        return Err(Error::MissingRequires("--baselines", "--compare"));
    } else if args.load_baseline.is_some() && args.baseline.is_none() {
        return Err(Error::MissingRequires("--load-baseline", "--baseline"));
    }

    // Error if there are conflicting args
    if args.verbose && args.quiet {
        return Err(Error::ConflictingFlags(&["--verbose", "--quiet"]));
    } else if [
        args.discard_baseline,
        !args.save_baseline.is_default(),
        args.baseline.is_some(),
        args.baseline_lenient.is_some(),
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
        args.list,
        args.test,
        args.profile_time.is_some(),
        args.export.is_some(),
        args.compare,
        args.load_baseline.is_some(),
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
    } else if args.quick && args.sample_size.is_some() {
        return Err(Error::ConflictingFlags(&["--quick", "--sample-size"]));
    }

    Ok(args)
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
