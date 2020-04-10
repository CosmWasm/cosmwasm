use std::env::current_dir;
use std::{fs, io, path};

use cosmwasm_schema::{export_schema, schema_for};

use reflect::msg::{HandleMsg, InitMsg, OwnerResponse, QueryMsg};
use reflect::state::State;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    clean_dir(&out_dir).unwrap();

    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(HandleMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(OwnerResponse), &out_dir);
}

// TODO: move this to cosmwasm-schema
fn clean_dir(out_dir: &path::Path) -> Result<(), io::Error> {
    fs::create_dir_all(out_dir).unwrap();
    let entries: Vec<_> = fs::read_dir(out_dir)?
        // we ignore read errors on entries
        .filter(|res| res.is_ok())
        .map(|res| res.unwrap().path())
        .filter(|res| res.is_file())
        .collect();

    for file in entries {
        fs::remove_file(file)?;
    }
    Ok(())
}
