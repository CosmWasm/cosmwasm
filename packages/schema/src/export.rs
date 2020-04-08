//! Export schema to file

use std::fs::write;
use std::path::PathBuf;

use schemars::schema::RootSchema;

use crate::casing::to_snake_case;

// Exports a schema, auto-generating filename based on the metadata title of the generated schema.
pub fn export_schema(schema: &RootSchema, out_dir: &PathBuf) -> () {
    let title = schema
        .schema
        .metadata
        .as_ref()
        .map(|b| b.title.clone().unwrap_or("untitled".to_string()))
        .unwrap_or("unknown".to_string());
    write_schema(schema, out_dir, &title);
}

// use this if you want to override the auto-detected name of the object.
// very useful when creating an alias for a type-alias.
pub fn export_schema_with_title(schema: &mut RootSchema, out_dir: &PathBuf, title: &str) -> () {
    // set the title explicitly on the schemas metadata
    let metadata = &mut schema.schema.metadata;
    if let Some(data) = metadata {
        data.title = Some(title.to_string());
    }
    write_schema(schema, out_dir, &title);
}

/// Writes schema to file. Overwrites existing file.
/// Panics on any error writing out the schema.
fn write_schema(schema: &RootSchema, out_dir: &PathBuf, title: &str) -> () {
    // first, we set the title as we wish
    let path = out_dir.join(format!("{}.json", to_snake_case(&title)));
    let json = serde_json::to_string_pretty(schema).unwrap();
    write(&path, json + "\n").unwrap();
    println!("Created {}", path.to_str().unwrap());
}
