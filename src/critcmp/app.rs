use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use regex::Regex;
use tabwriter::TabWriter;
use termcolor::{self, WriteColor};

use crate::critcmp::data::{BaseBenchmarks, Benchmarks};
use crate::critcmp::main::Result;

#[derive(Clone, Debug, Default)]
pub struct Args {
    pub baselines: Vec<String>,
    pub output_list: bool,
    pub threshold: Option<f64>,
    pub color: bool,
    pub filter: Option<Regex>,
}

impl Args {
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

    pub fn filter(&self) -> Option<&'_ Regex> {
        self.filter.as_ref()
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
    }

    pub fn list(&self) -> bool {
        self.output_list
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
        let mut cwd = fs::canonicalize(".")
            .ok()
            .unwrap_or_else(|| PathBuf::from("."));
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
