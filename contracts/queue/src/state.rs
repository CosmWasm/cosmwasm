use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// we store one entry for each item in the queue
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Item {
    pub value: i32,
}
