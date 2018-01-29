use std::fs::{self, File, ReadDir};
use std::io::Read;
use std::path::Path;
use serde_json;
use serde::Serialize;
use serde::de::DeserializeOwned;

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

pub fn ls(dir: &Path) -> Result<ReadDir> {
    Ok(fs::read_dir(dir)?)
}

pub fn mkdirp<P>(path: &P) -> Result<()>
where
    P: AsRef<Path>,
{
    Ok(fs::create_dir_all(path.as_ref())?)
}

pub fn mv(from: &Path, to: &Path) -> Result<()> {
    Ok(fs::rename(from, to)?)
}

pub fn rmrf(path: &Path) -> Result<()> {
    fs::remove_dir_all(path)?;

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
