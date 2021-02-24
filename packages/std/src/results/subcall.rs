use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Binary;

use super::Attribute;

/// The information we get back from the sub-call, with full sdk events
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubcallResponse {
    pub events: Vec<Event>,
    pub data: Option<Binary>,
}

/// a full sdk event
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Event {
    #[serde(rename = "type")]
    pub kind: String,
    pub attributes: Vec<Attribute>,
}
