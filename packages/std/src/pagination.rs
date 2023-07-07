use crate::Binary;

/// Simplified version of the PageRequest type for pagination from the cosmos-sdk
#[cosmwasm_schema::cw_serde_prost]
#[derive(Eq)]
pub struct PageRequest {
    #[prost(message, tag = "1")]
    pub key: Option<Binary>,
    #[prost(uint32, tag = "2")]
    pub limit: u32,
    #[prost(bool, tag = "3")]
    pub reverse: bool,
}
