use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct MigrateMsg {
    /// The address we send all remaining balance to
    pub payout: String,
    /// Optional amount of items to delete in this call.
    /// If it is not provided, nothing will be deleted.
    /// You can delete further items in a subsequent execute call.
    #[serde(default)]
    pub delete: u32,
}

/// A placeholder where we don't take any input
#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// Cleans up the given number of state elements.
    /// Call this multiple times to increamentally clean up state.
    Cleanup {
        /// The number of state elements to delete.
        ///
        /// Set this to None for unlimited cleanup (if your state is small or you are feeling YOLO)
        limit: Option<u32>,
    },
}
