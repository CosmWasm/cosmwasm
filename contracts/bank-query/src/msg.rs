use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Supply {
        denom: String
    },
    Balance {
        address: String,
        denom: String,
    },
    AllBalances {
        address: String,
    },
}
