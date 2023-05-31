use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Binary, Uint64};

/// Simplified version of the PageRequest type for pagination from the cosmos-sdk
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct PageRequest {
    pub key: Option<Binary>,
    pub limit: Uint64,
    pub reverse: bool,
}
