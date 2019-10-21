use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use failure::Error;
use sha2::{Digest, Sha256};

/// save stores the wasm code in the given directory and returns an ID for lookup.
/// It will create the directory if it doesn't exist.
/// If the file already exists, it will return an error.
pub fn save(dir: &str, wasm: &[u8]) -> Result<Vec<u8>, Error> {
    // create directory if needed
    let path = Path::new(dir);
    DirBuilder::new().recursive(true).create(path)?;

    // calculate filename
    let id = Sha256::digest(wasm).to_vec();
    let filename = hex::encode(&id);
    let filepath = path.join(&filename);

    // write data to file
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(filepath)?;
    file.write_all(wasm)?;

    Ok(id)
}

pub fn load(dir: &str, id: &[u8]) -> Result<Vec<u8>, Error> {
    // this requires the directory and file to exist
    let path = Path::new(dir).join(hex::encode(id));
    let mut file = File::open(path)?;

    let mut wasm = Vec::<u8>::new();
    let _ = file.read_to_end(&mut wasm)?;
    Ok(wasm)
}

#[cfg(test)]
mod test {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn save_and_load() {
        let tmp_dir = TempDir::new("comswasm_vm_test").unwrap();
        let path = tmp_dir.path().to_str().unwrap();
        let code = vec![12u8; 17];
        let id = save(path, &code).unwrap();
        assert_eq!(id.len(), 32);

        let loaded = load(path, &id).unwrap();
        assert_eq!(code, loaded);
    }

    #[test]
    fn fails_on_invalid_dir() {
        let path = "/foo/bar";
        let code = vec![12u8; 17];
        let id = save(path, &code);
        assert!(id.is_err());
    }

    #[test]
    fn file_already_exists() {
        let tmp_dir = TempDir::new("comswasm_vm_test").unwrap();
        let path = tmp_dir.path().to_str().unwrap();
        let code = vec![12u8; 17];
        let id = save(path, &code).unwrap();
        assert_eq!(id.len(), 32);

        let dup = save(path, &code);
        assert!(dup.is_err());
    }
}
