#[cfg(feature = "std")]
use {
    cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for},
    cosmwasm_std::{BlockInfo, CosmosMsg, Empty, QueryRequest, Timestamp},
    std::env::current_dir,
    std::fs::create_dir_all,
};

#[cfg(feature = "std")]
fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(BlockInfo), &out_dir);
    export_schema(&schema_for!(Timestamp), &out_dir);
    export_schema_with_title(&schema_for!(CosmosMsg), &out_dir, "CosmosMsg");
    export_schema_with_title(&schema_for!(QueryRequest<Empty>), &out_dir, "QueryRequest");
}

#[cfg(not(feature = "std"))]
fn main() {}
