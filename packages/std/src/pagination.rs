use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Binary, Uint64};

/// Replicates the PageRequest type for pagination from the cosmos-sdk
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct PageRequest {
    pub key: Binary,
    pub offset: Uint64,
    pub limit: Uint64,
    pub count_total: bool,
    pub reverse: bool,
}
