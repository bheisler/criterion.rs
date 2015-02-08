use rustc_serialize::json::Encoder;
use rustc_serialize::{Decodable, Encodable, json};
use std::old_io::{File, USER_RWX, fs};

// TODO Proper error handling
pub fn load<A: Decodable>(path: &Path) -> A {
    match File::open(path) {
        Err(e) => panic!("{}", e),
        Ok(mut f) => match f.read_to_string() {
            Err(e) => panic!("{}", e),
            Ok(s) => match json::decode(&*s) {
                Err(e) => panic!("Couldn't decode {} ({:?})", s, e),
                Ok(thing) => thing,
            }
        }
    }
}

pub fn ls(dir: &Path) -> Vec<Path> {
    match fs::readdir(dir) {
        Err(e) => panic!("`ls {}`: {}", dir.display(), e),
        Ok(contents) => contents,
    }
}

pub fn mkdirp(path: &Path) {
    if let Err(e) = fs::mkdir_recursive(path, USER_RWX) {
        panic!("`mkdir -p {}`: {}", path.display(), e)
    }
}

pub fn mv(from: &Path, to: &Path) {
    if let Err(e) = fs::rename(from, to) {
        panic!("`mv {} {}`: {}", from.display(), to.display(), e)
    }
}

pub fn rmrf(path: &Path) {
    if let Err(e) = fs::rmdir_recursive(path) {
        panic!("`rm -rf {}`: {}", path.display(), e)
    }
}

// TODO Proper error handling
pub fn save<D: Encodable>(data: &D, path: &Path) {
    let mut buf = String::new();
    {
        let ref mut encoder = Encoder::new_pretty(&mut buf);
        data.encode(encoder).unwrap();
    }
    File::create(path).write_all(buf.as_bytes()).ok().expect("Couldn't save data")
}
