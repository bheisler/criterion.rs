use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use regex::Regex;
use tabwriter::TabWriter;
use termcolor::{self, WriteColor};

use crate::critcmp::data::{BaseBenchmarks, Benchmarks};
use crate::critcmp::main::Result;

// const TEMPLATE: &'static str = "\
// {bin} {version}
// {author}
// {about}

// USAGE:
//     {usage}

// SUBCOMMANDS:
// {subcommands}

// OPTIONS:
// {unified}";

// const ABOUT: &'static str = "
// critcmp is a tool for comparing benchmark results produced by Criterion.

// critcmp works by slurping up all benchmark data from Criterion's target
// directory, in addition to extra data supplied as positional parameters. The
// primary unit that critcmp works with is Criterion's baselines. That is, the
// simplest way to use critcmp is to save two baselines with Criterion's benchmark
// harness and then compare them. For example:

//     $ cargo bench -- --save-baseline before
//     $ cargo bench -- --save-baseline change
//     $ critcmp before change

// Filtering can be done with the -f/--filter flag to limit comparisons based on
// a regex:

//     $ critcmp before change -f 'foo.*bar'

// Comparisons with very small differences can also be filtered out. For example,
// this hides comparisons with differences of 5% or less

//     $ critcmp before change -t 5

// Comparisons are not limited to only two baselines. Many can be used:

//     $ critcmp before change1 change2

// The list of available baselines known to critcmp can be printed:

//     $ critcmp --baselines

// A baseline can exported to one JSON file for more permanent storage outside
// of Criterion's target directory:

//     $ critcmp --export before > before.json
//     $ critcmp --export change > change.json

// Baselines saved this way can be used by simply using their file path instead
// of just the name:

//     $ critcmp before.json change.json

// Benchmarks within the same baseline can be compared as well. Normally,
// benchmarks are compared based on their name. That is, given two baselines, the
// correspondence between benchmarks is established by their name. Sometimes,
// however, you'll want to compare benchmarks that don't have the same name. This
// can be done by expressing the matching criteria via a regex. For example, given
// benchmarks 'optimized/input1' and 'naive/input1' in the baseline 'benches', the
// following will show a comparison between the two benchmarks despite the fact
// that they have different names:

//     $ critcmp benches -g '\\w+/(input1)'

// That is, the matching criteria is determined by the values matched by all of
// the capturing groups in the regex. All benchmarks with equivalent capturing
// groups will be included in one comparison. There is no limit on the number of
// benchmarks that can appear in a single comparison.

// Finally, if comparisons grow too large to see in the default column oriented
// display, then the results can be flattened into lists:

//     $ critcmp before change1 change2 change3 change4 change5 --list

// Project home page: https://github.com/BurntSushi/critcmp
// Criterion home page: https://github.com/japaric/criterion.rs";

#[derive(Clone, Debug)]
// pub struct Args(ArgMatches<'static>);
pub struct Args {
    pub baselines: Vec<String>,
    pub output_list: bool,
    pub threshold: Option<f64>,
    pub color: bool,
}

impl Args {
    // pub fn parse() -> Args {
    //     Args(app().get_matches())
    // }

    pub fn benchmarks(&self) -> Result<Benchmarks> {
        // First, load benchmark data from command line parameters. If a
        // baseline name is given and is not a file path, then it is added to
        // our whitelist of baselines.
        let mut from_cli: Vec<BaseBenchmarks> = vec![];
        let mut whitelist = BTreeSet::new();
        for arg in self.baselines.iter() {
            let p = Path::new(arg);
            if p.is_file() {
                let baseb = BaseBenchmarks::from_path(p)
                    .map_err(|err| format!("{}: {}", p.display(), err))?;
                whitelist.insert(baseb.name.clone());
                from_cli.push(baseb);
            } else {
                whitelist.insert(arg.clone());
            }
        }

        let mut from_crit: Vec<BaseBenchmarks> = vec![];
        match self.criterion_dir() {
            Err(err) => {
                // If we've loaded specific benchmarks from arguments, then it
                // shouldn't matter whether we can find a Criterion directory.
                // If we haven't loaded anything explicitly though, and if
                // Criterion detection fails, then we won't have loaded
                // anything and so we should return an error.
                if from_cli.is_empty() {
                    return Err(err);
                }
            }
            Ok(critdir) => {
                let data = Benchmarks::gather(critdir)?;
                from_crit.extend(data.by_baseline.into_iter().map(|(_, v)| v));
            }
        }
        if from_cli.is_empty() && from_crit.is_empty() {
            fail!("could not find any benchmark data");
        }

        let mut data = Benchmarks::default();
        for basebench in from_crit.into_iter().chain(from_cli) {
            if !whitelist.is_empty() && !whitelist.contains(&basebench.name) {
                continue;
            }
            data.by_baseline.insert(basebench.name.clone(), basebench);
        }
        Ok(data)
    }

    pub fn filter(&self) -> Result<Option<Regex>> {
        // TODO
        Ok(None)
        // let pattern_os = match self.0.value_of_os("filter") {
        //     None => return Ok(None),
        //     Some(pattern) => pattern,
        // };
        // let pattern = cli::pattern_from_os(pattern_os)?;
        // Ok(Some(Regex::new(pattern)?))
    }

    pub fn group(&self) -> Result<Option<Regex>> {
        // TODO
        Ok(None)
        // let pattern_os = match self.0.value_of_os("group") {
        //     None => return Ok(None),
        //     Some(pattern) => pattern,
        // };
        // let pattern = cli::pattern_from_os(pattern_os)?;
        // let re = Regex::new(pattern)?;
        // if re.captures_len() <= 1 {
        //     fail!(
        //         "pattern '{}' has no capturing groups, by grouping \
        //          benchmarks by a regex requires the use of at least \
        //          one capturing group",
        //         pattern
        //     );
        // }
        // Ok(Some(re))
    }

    pub fn threshold(&self) -> Result<Option<f64>> {
        Ok(self.threshold)
        // let percent = match self.0.value_of_lossy("threshold") {
        //     None => return Ok(None),
        //     Some(percent) => percent,
        // };
        // Ok(Some(percent.parse()?))
    }

    pub fn baselines(&self) -> bool {
        false
        // !self.baselines.is_empty()
    }

    pub fn list(&self) -> bool {
        self.output_list
    }

    pub fn export(&self) -> Option<String> {
        None
        // self.0.value_of_lossy("export").map(|v| v.into_owned())
    }

    pub fn criterion_dir(&self) -> Result<PathBuf> {
        let target_dir = self.target_dir()?;
        let crit_dir = target_dir.join("criterion");
        if !crit_dir.exists() {
            fail!(
                "\
                 no criterion data exists at {}\n\
                 try running your benchmarks before tabulating results\
                 ",
                crit_dir.display()
            );
        }
        Ok(crit_dir)
    }

    pub fn stdout(&self) -> Box<dyn WriteColor> {
        if self.color {
            Box::new(termcolor::Ansi::new(TabWriter::new(io::stdout())))
        } else {
            Box::new(termcolor::NoColor::new(TabWriter::new(io::stdout())))
        }
    }

    fn target_dir(&self) -> Result<PathBuf> {
        // FIXME: Use the same code as criterion
        let mut cwd = fs::canonicalize(".")?;
        loop {
            let candidate = cwd.join("target");
            if candidate.exists() {
                return Ok(candidate);
            }
            cwd = match cwd.parent() {
                Some(p) => p.to_path_buf(),
                None => {
                    fail!(
                        "\
                         could not find Criterion output directory\n\
                         try using --target-dir or set CARGO_TARGET_DIR\
                         "
                    );
                }
            }
        }
    }
}

// fn app() -> App<'static, 'static> {
//     // The actual App.
//     App::new("critcmp")
//         .author(crate_authors!())
//         .version(crate_version!())
//         .about(ABOUT)
//         .template(TEMPLATE)
//         .max_term_width(100)
//         .setting(AppSettings::UnifiedHelpMessage)
//         .arg(
//             Arg::with_name("target-dir")
//                 .long("target-dir")
//                 .takes_value(true)
//                 .env("CARGO_TARGET_DIR")
//                 .help(
//                     "The path to the target directory where Criterion's \
//                    benchmark data is stored.",
//                 ),
//         )
//         .arg(
//             Arg::with_name("baselines")
//                 .long("baselines")
//                 .help("List all available baselines."),
//         )
//         .arg(
//             Arg::with_name("export")
//                 .long("export")
//                 .takes_value(true)
//                 .help(
//                     "Export all of the benchmark data for a specific baseline \
//                    as JSON data printed to stdout. A file containing the data \
//                    written can be passed as a positional argument to critcmp \
//                    in order to load the baseline data.",
//                 ),
//         )
//         .arg(Arg::with_name("list").long("list").help(
//             "Show each benchmark comparison as a list. This is useful \
//                    when there are many comparisons for each benchmark such \
//                    that they no longer fit in a column view.",
//         ))
//         .arg(
//             Arg::with_name("filter")
//                 .long("filter")
//                 .short("f")
//                 .takes_value(true)
//                 .help(
//                     "Filter benchmarks by a regex. Benchmark names are given to \
//                    this regex. Matches are shown while non-matches are not.",
//                 ),
//         )
//         .arg(
//             Arg::with_name("group")
//                 .long("group")
//                 .short("g")
//                 .takes_value(true)
//                 .help(
//                     "Group benchmarks by a regex. This requires at least one \
//                    capturing group. All benchmarks whose capturing group \
//                    values match are compared with one another.",
//                 ),
//         )
//         .arg(
//             Arg::with_name("threshold")
//                 .long("threshold")
//                 .short("t")
//                 .takes_value(true)
//                 .help(
//                     "A threshold where by comparisons with differences below \
//                    this percentage are not shown. By default, all comparisons \
//                    are shown. Example use: '-t 5' hides any comparisons with \
//                    differences under 5%.",
//                 ),
//         )
//         .arg(
//             Arg::with_name("color")
//                 .long("color")
//                 .takes_value(true)
//                 .possible_values(&["never", "always", "auto"])
//                 .default_value("auto")
//                 .help(
//                     "Set whether color should or should not be shown. When \
//                    'auto' is used (the default), then color will only be used \
//                    when printing to a tty.",
//                 ),
//         )
//         .arg(Arg::with_name("args").multiple(true).help(
//             "A baseline name, file path to a baseline or a regex pattern
//                    for selecting benchmarks.",
//         ))
// }
