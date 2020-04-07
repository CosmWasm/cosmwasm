use std::env::current_dir;
use std::fs::{create_dir_all};

use cosmwasm_schema::{export_schema, schema_for};

use queue::contract::{CountResponse, HandleMsg, InitMsg, Item, QueryMsg, SumResponse};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();

    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(HandleMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Item), &out_dir);
    export_schema(&schema_for!(CountResponse), &out_dir);
    export_schema(&schema_for!(SumResponse), &out_dir);
}
