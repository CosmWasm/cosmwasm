use std::env::current_dir;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;

use schemars::{schema::RootSchema, schema_for};

use queue::contract::{CountResponse, HandleMsg, InitMsg, Item, QueryMsg, SumResponse};

fn main() {
    let mut pwd = current_dir().unwrap();
    pwd.push("schema");
    create_dir_all(&pwd).unwrap();

    let schema = schema_for!(InitMsg);
    export_schema(&schema, &pwd, "init_msg.json");

    let schema = schema_for!(HandleMsg);
    export_schema(&schema, &pwd, "handle_msg.json");

    let schema = schema_for!(QueryMsg);
    export_schema(&schema, &pwd, "query_msg.json");

    let schema = schema_for!(Item);
    export_schema(&schema, &pwd, "item.json");

    let schema = schema_for!(CountResponse);
    export_schema(&schema, &pwd, "count_response.json");

    let schema = schema_for!(SumResponse);
    export_schema(&schema, &pwd, "sum_response.json");
}

/// Writes schema to file. Overwrites existing file.
/// Panics on any error writing out the schema.
fn export_schema(schema: &RootSchema, out_dir: &PathBuf, file_name: &str) -> () {
    let path = out_dir.join(file_name);
    let json = serde_json::to_string_pretty(schema).unwrap();
    write(&path, json + "\n").unwrap();
    println!("Created {}", path.to_str().unwrap());
}
