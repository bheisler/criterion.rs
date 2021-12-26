use std::collections::BTreeMap;
use std::fs::File;
use std::io;
use std::path::Path;

use serde::de::DeserializeOwned;
// use serde::{Deserialize, Serialize};
use serde_json as json;
use walkdir::WalkDir;

use crate::critcmp::main::Result;

#[derive(Clone, Debug, Default)]
pub struct Benchmarks {
    pub by_baseline: BTreeMap<String, BaseBenchmarks>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BaseBenchmarks {
    pub name: String,
    pub benchmarks: BTreeMap<String, Benchmark>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Benchmark {
    pub baseline: String,
    pub fullname: String,
    #[serde(rename = "criterion_benchmark_v1")]
    pub info: CBenchmark,
    #[serde(rename = "criterion_estimates_v1")]
    pub estimates: CEstimates,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CBenchmark {
    pub group_id: String,
    pub function_id: Option<String>,
    pub value_str: Option<String>,
    pub throughput: Option<CThroughput>,
    pub full_id: String,
    pub directory_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CThroughput {
    pub bytes: Option<u64>,
    pub elements: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CEstimates {
    pub mean: CStats,
    pub median: CStats,
    pub median_abs_dev: CStats,
    pub slope: Option<CStats>,
    pub std_dev: CStats,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CStats {
    pub confidence_interval: CConfidenceInterval,
    pub point_estimate: f64,
    pub standard_error: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CConfidenceInterval {
    pub confidence_level: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
}

impl Benchmarks {
    pub fn gather<P: AsRef<Path>>(criterion_dir: P) -> Result<Benchmarks> {
        let mut benchmarks = Benchmarks::default();
        for result in WalkDir::new(criterion_dir) {
            let dent = result?;
            let b = match Benchmark::from_path(dent.path())? {
                None => continue,
                Some(b) => b,
            };
            benchmarks
                .by_baseline
                .entry(b.baseline.clone())
                .or_insert_with(|| BaseBenchmarks {
                    name: b.baseline.clone(),
                    benchmarks: BTreeMap::new(),
                })
                .benchmarks
                .insert(b.benchmark_name().to_string(), b);
        }
        Ok(benchmarks)
    }
}

impl Benchmark {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Option<Benchmark>> {
        let path = path.as_ref();
        Benchmark::from_path_imp(path).map_err(|err| {
            if let Some(parent) = path.parent() {
                err!("{}: {}", parent.display(), err)
            } else {
                err!("unknown path: {}", err)
            }
        })
    }

    fn from_path_imp(path: &Path) -> Result<Option<Benchmark>> {
        match path.file_name() {
            None => return Ok(None),
            Some(filename) => {
                if filename != "estimates.json" {
                    return Ok(None);
                }
            }
        }
        // Criterion's directory structure looks like this:
        //
        //     criterion/{group}/{name}/{baseline}/estimates.json
        //
        // In the same directory as `estimates.json`, there is also a
        // `benchmark.json` which contains most of the info we need about
        // a benchmark, including its name. From the path, we only extract the
        // baseline name.
        let parent = path
            .parent()
            .ok_or_else(|| err!("{}: could not find parent dir", path.display()))?;
        let baseline = parent
            .file_name()
            .map(|p| p.to_string_lossy().into_owned())
            .ok_or_else(|| err!("{}: could not find baseline name", path.display()))?;
        if baseline == "change" {
            // This isn't really a baseline, but special state emitted by
            // Criterion to reflect its own comparison between baselines. We
            // don't use it.
            return Ok(None);
        }

        let info = CBenchmark::from_path(parent.join("benchmark.json"))?;
        let estimates = CEstimates::from_path(path)?;
        let fullname = format!("{}/{}", baseline, info.full_id);
        Ok(Some(Benchmark {
            baseline,
            fullname,
            info,
            estimates,
        }))
    }

    pub fn nanoseconds(&self) -> f64 {
        self.estimates.mean.point_estimate
    }

    pub fn stddev(&self) -> f64 {
        self.estimates.std_dev.point_estimate
    }

    pub fn fullname(&self) -> &str {
        &self.fullname
    }

    pub fn baseline(&self) -> &str {
        &self.baseline
    }

    pub fn benchmark_name(&self) -> &str {
        &self.info.full_id
    }

    pub fn throughput(&self) -> Option<Throughput> {
        const NANOS_PER_SECOND: f64 = 1_000_000_000.0;

        let scale = NANOS_PER_SECOND / self.nanoseconds();

        self.info.throughput.as_ref().and_then(|t| {
            #[allow(clippy::manual_map)]
            if let Some(num) = t.bytes {
                Some(Throughput::Bytes(num as f64 * scale))
            } else if let Some(num) = t.elements {
                Some(Throughput::Elements(num as f64 * scale))
            } else {
                None
            }
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Throughput {
    Bytes(f64),
    Elements(f64),
}

impl BaseBenchmarks {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<BaseBenchmarks> {
        deserialize_json_path(path.as_ref())
    }
}

impl CBenchmark {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<CBenchmark> {
        deserialize_json_path(path.as_ref())
    }
}

impl CEstimates {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<CEstimates> {
        deserialize_json_path(path.as_ref())
    }
}

fn deserialize_json_path<D: DeserializeOwned>(path: &Path) -> Result<D> {
    let file = File::open(path).map_err(|err| {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            err!("{}: {}", name, err)
        } else {
            err!("{}: {}", path.display(), err)
        }
    })?;
    let buf = io::BufReader::new(file);
    let b = json::from_reader(buf).map_err(|err| {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            err!("{}: {}", name, err)
        } else {
            err!("{}: {}", path.display(), err)
        }
    })?;
    Ok(b)
}
