use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for, export_schema_with_title};

use ibc_reflect::msg::{HandleMsg, InitMsg, PacketMsg, AcknowledgementMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(HandleMsg), &out_dir);
    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(PacketMsg), &out_dir);
    export_schema_with_title(&mut schema_for!(AcknowledgementMsg), &out_dir, "AcknowledgementMsg");
}
