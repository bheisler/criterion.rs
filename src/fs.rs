use std::io::fs;
use std::io::{UserRWX};

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
        Err(e) => fail!("`rm -rf {}: {}`", path.display(), e),
        Ok(_) => {},
    }
}
