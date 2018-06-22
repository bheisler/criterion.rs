use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use error::{AccessError, Result};

pub fn load<A, P: ?Sized>(path: &P) -> Result<A>
where
    A: DeserializeOwned,
    P: AsRef<Path>,
{
    let mut f = File::open(path).map_err(|inner| AccessError {
        inner,
        path: path.as_ref().to_owned(),
    })?;
    let mut string = String::new();
    let _ = f.read_to_string(&mut string);
    let result: A = serde_json::from_str(string.as_str())?;

    Ok(result)
}

pub fn mkdirp<P>(path: &P) -> Result<()>
where
    P: AsRef<Path>,
{
    fs::create_dir_all(path.as_ref())?;
    Ok(())
}

pub fn cp(from: &Path, to: &Path) -> Result<()> {
    fs::copy(from, to)?;
    Ok(())
}

pub fn save<D, P>(data: &D, path: &P) -> Result<()>
where
    D: Serialize,
    P: AsRef<Path>,
{
    let buf = serde_json::to_string(&data)?;
    save_string(&buf, path)
}

pub fn save_string<P>(data: &str, path: &P) -> Result<()>
where
    P: AsRef<Path>,
{
    use std::io::Write;

    File::create(path)
        .and_then(|mut f| f.write_all(data.as_bytes()))
        .map_err(|inner| AccessError {
            inner,
            path: path.as_ref().to_owned(),
        })?;

    Ok(())
}

pub fn list_existing_reports<P>(directory: &P) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let mut paths = vec![];
    for entry in fs::read_dir(directory)? {
        let path = entry?.path().join("report");
        if path.is_dir() && path.join("index.html").is_file() {
            paths.push(path.to_owned());
        }
    }
    Ok(paths)
}
