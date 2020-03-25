use std::env::current_dir;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;

use schemars::{schema::RootSchema, schema_for};

use queue::contract::{CountResponse, HandleMsg, InitMsg, QueryMsg, State, SumResponse};

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

    let schema = schema_for!(State);
    export_schema(&schema, &pwd, "state.json");

    let schema = schema_for!(CountResponse);
    export_schema(&schema, &pwd, "count_response.json");

    let schema = schema_for!(SumResponse);
    export_schema(&schema, &pwd, "sum_response.json");
}

// panics if any error writing out the schema
// overwrites any existing schema
fn export_schema(schema: &RootSchema, dir: &PathBuf, name: &str) -> () {
    let path = dir.join(name);
    let json = serde_json::to_string_pretty(schema).unwrap();
    write(&path, json.as_bytes()).unwrap();
    println!("{}", path.to_str().unwrap());
}
