use serialize::json::PrettyEncoder;
use serialize::{Decodable, Encodable, json};
use std::io::{File, IoError, MemWriter, UserRWX, fs};
use std::mem;

// TODO Proper error handling
pub fn load<A: Decodable<json::Decoder, json::DecoderError>>(path: &Path) -> A {
    json::decode(File::open(path).read_to_string().ok().expect("Couldn't open file").as_slice()).
        ok().expect("Couldn't decode data")
}

pub fn ls(dir: &Path) -> Vec<Path> {
    match fs::readdir(dir) {
        Err(e) => fail!("`ls {}`: {}", dir.display(), e),
        Ok(contents) => contents,
    }
}

pub fn mkdirp(path: &Path) {
    match fs::mkdir_recursive(path, UserRWX) {
        Err(e) => fail!("`mkdir -p {}`: {}", path.display(), e),
        Ok(_) => {},
    }
}

pub fn mv(from: &Path, to: &Path) {
    match fs::rename(from, to) {
        Err(e) => fail!("`mv {} {}`: {}", from.display(), to.display(), e),
        Ok(_) => {},
    }
}

pub fn rmrf(path: &Path) {
    match fs::rmdir_recursive(path) {
        Err(e) => fail!("`rm -rf {}`: {}", path.display(), e),
        Ok(_) => {},
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
    File::create(path).write(writer.get_ref()).ok().
        expect("Couldn't save data")
}
