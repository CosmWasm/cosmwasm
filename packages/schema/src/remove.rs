use std::{fs, io, path};

fn is_regular_file(path: &path::Path) -> Result<bool, io::Error> {
    Ok(path.symlink_metadata()?.is_file())
}

fn is_hidden(path: &path::Path) -> bool {
    match path.file_name() {
        Some(name) => name.to_os_string().to_string_lossy().starts_with('.'),
        None => false, // a path without filename is no .*
    }
}

fn is_json(path: &path::Path) -> bool {
    match path.file_name() {
        Some(name) => name.to_os_string().to_string_lossy().ends_with(".json"),
        None => false, // a path without filename is no *.json
    }
}

pub fn remove_schemas(schemas_dir: &path::Path) -> Result<(), io::Error> {
    let file_paths = fs::read_dir(schemas_dir)?
        .filter_map(Result::ok) // skip read errors on entries
        .map(|entry| entry.path())
        .filter(|path| is_regular_file(path).unwrap_or(false)) // skip directories and symlinks
        .filter(|path| !is_hidden(path)) // skip hidden
        .filter(|path| is_json(path)) // skip non JSON
        ;

    for file_path in file_paths {
        println!("Removing {:?} …", file_path);
        fs::remove_file(file_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ffi::OsStr;
    use std::path::Path;

    #[test]
    fn is_hidden_works() {
        assert_eq!(is_hidden(Path::new("/foo")), false);
        assert_eq!(is_hidden(Path::new("/foo/bar")), false);
        assert_eq!(is_hidden(Path::new("/foo/bar.txt")), false);
        assert_eq!(is_hidden(Path::new("~foo")), false);
        assert_eq!(is_hidden(Path::new("foo")), false);

        assert_eq!(is_hidden(Path::new("/.foo")), true);
        assert_eq!(is_hidden(Path::new("/foo/.bar")), true);
        assert_eq!(is_hidden(Path::new("/foo/.bar.txt")), true);
        assert_eq!(is_hidden(Path::new(".foo")), true);

        // no filename
        assert_eq!(is_hidden(Path::new("/")), false);
        assert_eq!(is_hidden(Path::new("")), false);

        // invalid UTF-8
        #[cfg(any(unix, target_os = "redox"))]
        {
            use std::os::unix::ffi::OsStrExt;
            let non_hidden = OsStr::from_bytes(&[0x66, 0x6f, 0x80, 0x6f]); // fo�o
            assert_eq!(is_hidden(Path::new(non_hidden)), false);
            let hidden = OsStr::from_bytes(&[0x2e, 0x66, 0x6f, 0x80, 0x6f]); // .fo�o
            assert_eq!(is_hidden(Path::new(hidden)), true);
        }
    }

    #[test]
    fn is_json_works() {
        assert_eq!(is_json(Path::new("/foo")), false);
        assert_eq!(is_json(Path::new("/foo/bar")), false);
        assert_eq!(is_json(Path::new("/foo/bar.txt")), false);
        assert_eq!(is_json(Path::new("~foo")), false);
        assert_eq!(is_json(Path::new("foo")), false);
        assert_eq!(is_json(Path::new("foo.json5")), false);

        assert_eq!(is_json(Path::new("/.json")), true);
        assert_eq!(is_json(Path::new("/foo/.bar.json")), true);
        assert_eq!(is_json(Path::new("/foo/bar.json")), true);
        assert_eq!(is_json(Path::new("foo.json")), true);

        // no filename
        assert_eq!(is_json(Path::new("/")), false);
        assert_eq!(is_json(Path::new("")), false);

        // invalid UTF-8
        #[cfg(any(unix, target_os = "redox"))]
        {
            use std::os::unix::ffi::OsStrExt;
            let non_hidden = OsStr::from_bytes(&[0x66, 0x6f, 0x80, 0x6f]); // fo�o
            assert_eq!(is_json(Path::new(non_hidden)), false);
            let hidden = OsStr::from_bytes(&[0x66, 0x6f, 0x80, 0x6f, 0x2e, 0x6a, 0x73, 0x6f, 0x6e]); // fo�o.json
            assert_eq!(is_json(Path::new(hidden)), true);
        }
    }
}
