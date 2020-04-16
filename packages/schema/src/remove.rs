use std::{fs, io, path};

fn is_regular_file(path: &path::Path) -> Result<bool, io::Error> {
    Ok(path.symlink_metadata()?.is_file())
}

fn is_hidden(path: &path::Path) -> bool {
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("")
        .starts_with('.')
}

fn is_json(path: &path::Path) -> bool {
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("")
        .ends_with(".json")
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
