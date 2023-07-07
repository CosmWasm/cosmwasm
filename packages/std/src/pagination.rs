use crate::Binary;

/// Simplified version of the PageRequest type for pagination from the cosmos-sdk
#[cosmwasm_schema::cw_serde]
#[derive(Eq)]
pub struct PageRequest {
    pub key: Option<Binary>,
    pub limit: u32,
    pub reverse: bool,
}
