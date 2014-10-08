use serialize::json::PrettyEncoder;
use serialize::{Decodable, Encodable, json};
use std::io::{File, IoError, MemWriter, USER_RWX, fs};
use std::mem;

// TODO Proper error handling
pub fn load<A: Decodable<json::Decoder, json::DecoderError>>(path: &Path) -> A {
    match File::open(path) {
        Err(e) => fail!("{}", e),
        Ok(mut f) => match f.read_to_string() {
            Err(e) => fail!("{}", e),
            Ok(s) => match json::decode(s[]) {
                Err(e) => fail!("Couldn't decode {} ({})", s, e),
                Ok(thing) => thing,
            }
        }
    }
}

pub fn ls(dir: &Path) -> Vec<Path> {
    match fs::readdir(dir) {
        Err(e) => fail!("`ls {}`: {}", dir.display(), e),
        Ok(contents) => contents,
    }
}

pub fn mkdirp(path: &Path) {
    if let Err(e) = fs::mkdir_recursive(path, USER_RWX) {
        fail!("`mkdir -p {}`: {}", path.display(), e)
    }
}

pub fn mv(from: &Path, to: &Path) {
    if let Err(e) = fs::rename(from, to) {
        fail!("`mv {} {}`: {}", from.display(), to.display(), e)
    }
}

pub fn rmrf(path: &Path) {
    if let Err(e) = fs::rmdir_recursive(path) {
        fail!("`rm -rf {}`: {}", path.display(), e)
    }
}

// TODO Proper error handling
pub fn save<'a, D: Encodable<json::PrettyEncoder<'a>, IoError>>(data: &D, path: &Path) {
    let mut writer = MemWriter::new();
    {
        let ref mut encoder = PrettyEncoder::new(&mut writer);
        // FIXME (rust-lang/rust#14302) Remove transmute
        data.encode(unsafe { mem::transmute(encoder) }).unwrap();
    }
    File::create(path).write(writer.get_ref()).ok().expect("Couldn't save data")
}
