use std::env::current_dir;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;

use schemars::{schema::RootSchema, schema_for};

use cosmwasm_std::{CosmosMsg, Env, HandleResult, InitResult, QueryResult};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();

    export_schema(&schema_for!(Env), &out_dir);
    export_schema(&schema_for!(CosmosMsg), &out_dir);
    export_schema_with_title(&mut schema_for!(InitResult), &out_dir, "InitResult");
    export_schema_with_title(&mut schema_for!(HandleResult), &out_dir, "HandleResult");
    export_schema_with_title(&mut schema_for!(QueryResult), &out_dir, "QueryResult");
}

// Exports a schema, auto-generating filename based on the metadata title of the generated schema.
fn export_schema(schema: &RootSchema, out_dir: &PathBuf) -> () {
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
fn export_schema_with_title(schema: &mut RootSchema, out_dir: &PathBuf, title: &str) -> () {
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

fn to_snake_case(name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in name.char_indices() {
        if index != 0 && ch.is_uppercase() {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}
