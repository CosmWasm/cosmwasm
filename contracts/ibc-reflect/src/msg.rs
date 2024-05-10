use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, CosmosMsg, IbcAcknowledgement, Uint64};

/// Just needs to know the code_id of a reflect contract to spawn sub-accounts
#[cw_serde]
pub struct InstantiateMsg {
    pub reflect_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    AsyncAck {
        /// Existing channel where the packet was received
        channel_id: String,
        /// Sequence number of the packet that was received
        packet_sequence: Uint64,
        /// The acknowledgement to send back
        ack: IbcAcknowledgement,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns (reflect) account that is attached to this channel,
    /// or none.
    #[returns(AccountResponse)]
    Account { channel_id: String },
    /// Returns all (channel, reflect_account) pairs.
    /// No pagination - this is a test contract
    #[returns(ListAccountsResponse)]
    ListAccounts {},
}

#[cw_serde]
pub struct AccountResponse {
    pub account: Option<String>,
}

#[cw_serde]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountInfo>,
}

#[cw_serde]
pub struct AccountInfo {
    pub account: String,
    pub channel_id: String,
}

#[cw_serde]
pub enum ReflectExecuteMsg {
    ReflectMsg { msgs: Vec<CosmosMsg> },
}

#[cw_serde]
pub enum PacketMsg {
    Dispatch { msgs: Vec<CosmosMsg> },
    WhoAmI {},
    Balances {},
    Panic {},
    ReturnErr { text: String },
    ReturnMsgs { msgs: Vec<CosmosMsg> },
    NoAck {},
}

/// A custom acknowledgement type.
/// The success type `T` depends on the PacketMsg variant.
///
/// This could be refactored to use [StdAck] at some point. However,
/// it has a different success variant name ("ok" vs. "result") and
/// a JSON payload instead of a binary payload.
///
/// [StdAck]: https://github.com/CosmWasm/cosmwasm/issues/1512
#[cw_serde]
pub enum AcknowledgementMsg<S> {
    Ok(S),
    Error(String),
}

impl<S> AcknowledgementMsg<S> {
    pub fn unwrap(self) -> S {
        match self {
            AcknowledgementMsg::Ok(data) => data,
            AcknowledgementMsg::Error(err) => panic!("{}", err),
        }
    }

    pub fn unwrap_err(self) -> String {
        match self {
            AcknowledgementMsg::Ok(_) => panic!("not an error"),
            AcknowledgementMsg::Error(err) => err,
        }
    }
}

/// This is the success response we send on ack for PacketMsg::Dispatch.
/// Just acknowledge success or error
pub type DispatchResponse = ();

/// This is the success response we send on ack for PacketMsg::WhoAmI.
/// Return the caller's account address on the remote chain
#[cw_serde]
pub struct WhoAmIResponse {
    pub account: String,
}

/// This is the success response we send on ack for PacketMsg::Balance.
/// Just acknowledge success or error
#[cw_serde]
pub struct BalancesResponse {
    pub account: String,
    pub balances: Vec<Coin>,
}

/// This is the success response we send on ack for PacketMsg::ReturnMsgs.
/// Just acknowledge success or error
pub type ReturnMsgsResponse = ();
