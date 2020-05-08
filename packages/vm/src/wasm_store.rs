use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use snafu::ResultExt;

use crate::checksum::Checksum;
use crate::errors::{IoErr, VmResult};

/// save stores the wasm code in the given directory and returns an ID for lookup.
/// It will create the directory if it doesn't exist.
/// Saving the same byte code multiple times is allowed.
pub fn save<P: Into<PathBuf>>(dir: P, wasm: &[u8]) -> VmResult<Checksum> {
    // calculate filename
    let checksum = Checksum::generate(wasm);
    let filename = hex::encode(&checksum.0);
    let filepath = dir.into().join(&filename);

    // write data to file
    // Since the same filename (a collision resistent hash) cannot be generated from two different byte codes
    // (even if a malicious actor tried), it is safe to override.
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(filepath)
        .context(IoErr {})?;
    file.write_all(wasm).context(IoErr {})?;

    Ok(checksum)
}

pub fn load<P: Into<PathBuf>>(dir: P, checksum: &Checksum) -> VmResult<Vec<u8>> {
    // this requires the directory and file to exist
    let path = dir.into().join(hex::encode(checksum.0));
    let mut file = File::open(path).context(IoErr {})?;

    let mut wasm = Vec::<u8>::new();
    let _ = file.read_to_end(&mut wasm).context(IoErr {})?;
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

        let loaded = load(path, &id).unwrap();
        assert_eq!(code, loaded);
    }

    #[test]
    fn save_same_data_multiple_times() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();
        let code = vec![12u8; 17];

        save(path, &code).unwrap();
        save(path, &code).unwrap();
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

        let loaded = load(&path, &id).unwrap();
        assert_eq!(code, loaded);
    }
}
