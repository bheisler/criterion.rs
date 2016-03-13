use std::fs::{File, ReadDir, self};
use std::path::Path;

use rustc_serialize::json::Encoder;
use rustc_serialize::{Decodable, Encodable, json};

// TODO Proper error handling
pub fn load<A, P: ?Sized>(path: &P) -> A where
    A: Decodable,
    P: AsRef<Path>,
{
    fn load_<A>(path: &Path) -> A where
        A: Decodable,
    {
        use std::io::Read;

        let mut string = String::new();

        match File::open(path) {
            Err(e) => panic!("{}", e),
            Ok(mut f) => match f.read_to_string(&mut string) {
                Err(e) => panic!("{}", e),
                Ok(_) => match json::decode(&*string) {
                    Err(e) => panic!("Couldn't decode {} ({:?})", string, e),
                    Ok(thing) => thing,
                }
            }
        }
    }

    load_(path.as_ref())
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
    D: Encodable,
    P: AsRef<Path>,
{
    fn save_<D>(data: &D, path: &Path) where
        D: Encodable,
    {
        use std::io::Write;

        let mut buf = String::new();
        {
            let encoder = &mut Encoder::new_pretty(&mut buf);
            data.encode(encoder).unwrap();
        }
        File::create(path).unwrap().write_all(buf.as_bytes()).expect("Couldn't save data")
    }

    save_(data, path.as_ref())
}
