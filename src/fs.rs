use std::fs::{File, ReadDir, self};
use std::path::{AsPath, Path};

use rustc_serialize::json::Encoder;
use rustc_serialize::{Decodable, Encodable, json};

// TODO Proper error handling
pub fn load<A: Decodable, P: AsPath + ?Sized>(path: &P) -> A {
    fn load_<A: Decodable>(path: &Path) -> A {
        use std::io::Read;

        let mut string = String::new();

        match File::open(path) {
            Err(e) => panic!("{}", e),
            Ok(mut f) => match f.read_to_string(&mut string) {
                Err(e) => panic!("{}", e),
                Ok(()) => match json::decode(&*string) {
                    Err(e) => panic!("Couldn't decode {} ({:?})", string, e),
                    Ok(thing) => thing,
                }
            }
        }
    }

    load_(path.as_path())
}

pub fn ls(dir: &Path) -> ReadDir {
    match fs::read_dir(dir) {
        Err(e) => panic!("`ls {}`: {}", dir.display(), e),
        Ok(contents) => contents,
    }
}

pub fn mkdirp<P: AsPath>(path: &P) {
    fn mkdirp_(path: &Path) {
        if let Err(e) = fs::create_dir_all(path) {
            panic!("`mkdir -p {}`: {}", path.display(), e)
        }
    }

    mkdirp_(path.as_path())
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
pub fn save<D: Encodable, P: AsPath>(data: &D, path: &P) {
    fn save_<D: Encodable>(data: &D, path: &Path) {
        use std::io::Write;

        let mut buf = String::new();
        {
            let ref mut encoder = Encoder::new_pretty(&mut buf);
            data.encode(encoder).unwrap();
        }
        File::create(path).unwrap().write_all(buf.as_bytes()).ok().expect("Couldn't save data")
    }

    save_(data, path.as_path())
}
