use cosmwasm_schema::{schema_for, Api};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tempfile::tempfile;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub cap: u128,
}

// failure modes to help test wasmd, based on this comment
// https://github.com/cosmwasm/wasmd/issues/8#issuecomment-576146751
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint { amount: u128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Balance { account: String },
}

#[test]
fn test() -> anyhow::Result<()> {
    let file = tempfile()?;

    let mut api = Api {
        instantiate: schema_for!(InstantiateMsg),
        execute: schema_for!(ExecuteMsg),
        query: schema_for!(QueryMsg),
        //response: schema_for!(QueryResponse),
    }
    .render();
    let api = serde_json::to_writer_pretty(file, &api)?;

    // let mut out_dir = current_dir().unwrap();
    // out_dir.push("schema");
    // create_dir_all(&out_dir).unwrap();
    // remove_schemas(&out_dir).unwrap();

    // api.set_names();

    // // TODO: expose write_schema in export.rs (cosmwasm-schema)
    // let path = out_dir.join("api.json".to_string());
    // let json = serde_json::to_string_pretty(&api).unwrap();
    // write(&path, json + "\n").unwrap();
    // println!("Created {}", path.to_str().unwrap());
    Ok(())
}
