use std::fs::{File, ReadDir, self};
use std::io::Read;
use std::path::Path;
use serde_json;
use serde::Serialize;
use serde::de::DeserializeOwned;

// TODO Proper error handling
pub fn load<A, P: ?Sized>(path: &P) -> A where
    A: DeserializeOwned,
    P: AsRef<Path>,
{

    let mut f = File::open(path).unwrap_or_else(|e| { panic!("{}", e) });
    let mut string = String::new();
    let _ = f.read_to_string(&mut string);
    let result: A = serde_json::from_str(string.as_str()).unwrap();
    result
}

pub fn ls(dir: &Path) -> ReadDir {
    match fs::read_dir(dir) {
        Err(e) => panic!("`ls {}`: {}", dir.display(), e),
        Ok(contents) => contents,
    }
}

pub fn mkdirp<P>(path: &P) where
    P: AsRef<Path>,
{
    fn mkdirp_(path: &Path) {
        if let Err(e) = fs::create_dir_all(path) {
            panic!("`mkdir -p {}`: {}", path.display(), e)
        }
    }

    mkdirp_(path.as_ref())
}

pub fn mv(from: &Path, to: &Path) {
    if let Err(e) = fs::rename(from, to) {
        panic!("`mv {} {}`: {}", from.display(), to.display(), e)
    }
}

pub fn rmrf(path: &Path) {
    if let Err(e) = fs::remove_dir_all(path) {
        panic!("`rm -rf {}`: {}", path.display(), e)
    }
}

// TODO Proper error handling
pub fn save<D, P>(data: &D, path: &P) where
    D: Serialize,
    P: AsRef<Path>,
{
    fn save_<D>(data: &D, path: &Path) where
        D: Serialize,
    {
        use std::io::Write;

        let buf = serde_json::to_string(&data).unwrap();
        File::create(path).unwrap().write_all(buf.as_bytes()).expect("Couldn't save data")
    }

    save_(data, path.as_ref())
}
