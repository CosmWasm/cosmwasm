use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use snafu::ResultExt;

use crate::errors::{Error, IoErr};

pub fn wasm_hash(wasm: &[u8]) -> Vec<u8> {
    Sha256::digest(wasm).to_vec()
}

/// save stores the wasm code in the given directory and returns an ID for lookup.
/// It will create the directory if it doesn't exist.
/// If the file already exists, it will return an error.
pub fn save<P: Into<PathBuf>>(dir: P, wasm: &[u8]) -> Result<Vec<u8>, Error> {
    // calculate filename
    let id = wasm_hash(wasm);
    let filename = hex::encode(&id);
    let filepath = dir.into().join(&filename);

    // write data to file
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(filepath).context(IoErr{})?;
    file.write_all(wasm).context(IoErr{})?;

    Ok(id)
}

pub fn load<P: Into<PathBuf>>(dir: P, id: &[u8]) -> Result<Vec<u8>, Error> {
    // this requires the directory and file to exist
    let path = dir.into().join(hex::encode(id));
    let mut file = File::open(path).context(IoErr{})?;

    let mut wasm = Vec::<u8>::new();
    let _ = file.read_to_end(&mut wasm).context(IoErr{})?;
    Ok(wasm)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::create_dir_all;
    use tempfile::TempDir;

    #[test]
    fn save_and_load() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();
        let code = vec![12u8; 17];
        let id = save(path, &code).unwrap();
        assert_eq!(id.len(), 32);

        let loaded = load(path, &id).unwrap();
        assert_eq!(code, loaded);
    }

    #[test]
    fn fails_on_non_existent_dir() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("something");
        let code = vec![12u8; 17];
        let res = save(path.to_str().unwrap(), &code);
        assert!(res.is_err());
    }

    #[test]
    fn ensure_dir_prepares_space() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("something");
        create_dir_all(&path).unwrap();
        let code = vec![12u8; 17];
        let id = save(&path, &code).unwrap();
        assert_eq!(id.len(), 32);

        let loaded = load(&path, &id).unwrap();
        assert_eq!(code, loaded);
    }

    #[test]
    fn file_already_exists() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_str().unwrap();
        let code = vec![12u8; 17];
        let id = save(path, &code).unwrap();
        assert_eq!(id.len(), 32);

        let dup = save(path, &code);
        assert!(dup.is_err());
    }
}
