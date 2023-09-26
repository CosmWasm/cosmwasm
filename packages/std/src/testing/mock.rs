use serde::de::DeserializeOwned;
#[cfg(feature = "stargate")]
use serde::Serialize;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::addresses::{Addr, CanonicalAddr};
use crate::binary::Binary;
use crate::coin::Coin;
use crate::deps::OwnedDeps;
use crate::errors::{
    RecoverPubkeyError, SigningError, StdError, StdResult, SystemError, VerificationError,
};
#[cfg(feature = "stargate")]
use crate::ibc::{
    IbcAcknowledgement, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcEndpoint, IbcOrder, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcTimeoutBlock,
};
use crate::math::Uint128;
#[cfg(feature = "cosmwasm_1_1")]
use crate::query::SupplyResponse;
use crate::query::{
    AllBalanceResponse, BalanceResponse, BankQuery, CustomQuery, QueryRequest, WasmQuery,
};
#[cfg(feature = "staking")]
use crate::query::{
    AllDelegationsResponse, AllValidatorsResponse, BondedDenomResponse, DelegationResponse,
    FullDelegation, StakingQuery, Validator, ValidatorResponse,
};
use crate::results::{ContractResult, Empty, SystemResult};
use crate::serde::{from_slice, to_binary};
use crate::storage::MemoryStorage;
use crate::timestamp::Timestamp;
use crate::traits::{Api, Querier, QuerierResult};
use crate::types::{BlockInfo, ContractInfo, Env, MessageInfo, TransactionInfo};
use crate::Attribute;

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

/// Creates all external requirements that can be injected for unit tests.
///
/// See also [`mock_dependencies_with_balance`] and [`mock_dependencies_with_balances`]
/// if you want to start with some initial balances.
pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData,
    }
}

/// Creates all external requirements that can be injected for unit tests.
///
/// It sets the given balance for the contract itself, nothing else.
pub fn mock_dependencies_with_balance(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    mock_dependencies_with_balances(&[(MOCK_CONTRACT_ADDR, contract_balance)])
}

/// Initializes the querier along with the mock_dependencies.
/// Sets all balances provided (you must explicitly set contract balance if desired).
pub fn mock_dependencies_with_balances(
    balances: &[(&str, &[Coin])],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::new(balances),
        custom_query_type: PhantomData,
    }
}

// Use MemoryStorage implementation (which is valid in non-testcode)
// We can later make simplifications here if needed
pub type MockStorage = MemoryStorage;

/// Length of canonical addresses created with this API. Contracts should not make any assumtions
/// what this value is.
/// The value here must be restorable with `SHUFFLES_ENCODE` + `SHUFFLES_DECODE` in-shuffles.
const CANONICAL_LENGTH: usize = 54;

const SHUFFLES_ENCODE: usize = 18;
const SHUFFLES_DECODE: usize = 2;

// MockPrecompiles zero pads all human addresses to make them fit the canonical_length
// it trims off zeros for the reverse operation.
// not really smart, but allows us to see a difference (and consistent length for canonical adddresses)
#[derive(Copy, Clone)]
pub struct MockApi {
    /// Length of canonical addresses created with this API. Contracts should not make any assumtions
    /// what this value is.
    canonical_length: usize,
}

impl Default for MockApi {
    fn default() -> Self {
        MockApi {
            canonical_length: CANONICAL_LENGTH,
        }
    }
}

impl Api for MockApi {
    fn addr_validate(&self, input: &str) -> StdResult<Addr> {
        let canonical = self.addr_canonicalize(input)?;
        let normalized = self.addr_humanize(&canonical)?;
        if input != normalized {
            return Err(StdError::generic_err(
                "Invalid input: address not normalized",
            ));
        }

        Ok(Addr::unchecked(input))
    }

    fn addr_canonicalize(&self, input: &str) -> StdResult<CanonicalAddr> {
        // Dummy input validation. This is more sophisticated for formats like bech32, where format and checksum are validated.
        if input.len() < 3 {
            return Err(StdError::generic_err(
                "Invalid input: human address too short",
            ));
        }
        if input.len() > self.canonical_length {
            return Err(StdError::generic_err(
                "Invalid input: human address too long",
            ));
        }

        // mimicks formats like hex or bech32 where different casings are valid for one address
        let normalized = input.to_lowercase();

        let mut out = Vec::from(normalized);

        // pad to canonical length with NULL bytes
        out.resize(self.canonical_length, 0x00);
        // content-dependent rotate followed by shuffle to destroy
        // the most obvious structure (https://github.com/CosmWasm/cosmwasm/issues/552)
        let rotate_by = digit_sum(&out) % self.canonical_length;
        out.rotate_left(rotate_by);
        for _ in 0..SHUFFLES_ENCODE {
            out = riffle_shuffle(&out);
        }
        Ok(out.into())
    }

    fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr> {
        if canonical.len() != self.canonical_length {
            return Err(StdError::generic_err(
                "Invalid input: canonical address length not correct",
            ));
        }

        let mut tmp: Vec<u8> = canonical.clone().into();
        // Shuffle two more times which restored the original value (24 elements are back to original after 20 rounds)
        for _ in 0..SHUFFLES_DECODE {
            tmp = riffle_shuffle(&tmp);
        }
        // Rotate back
        let rotate_by = digit_sum(&tmp) % self.canonical_length;
        tmp.rotate_right(rotate_by);
        // Remove NULL bytes (i.e. the padding)
        let trimmed = tmp.into_iter().filter(|&x| x != 0x00).collect();
        // decode UTF-8 bytes into string
        let human = String::from_utf8(trimmed)?;
        Ok(Addr::unchecked(human))
    }

    fn secp256k1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Ok(secret_cosmwasm_crypto::secp256k1_verify(
            message_hash,
            signature,
            public_key,
        )?)
    }

    fn secp256k1_recover_pubkey(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        recovery_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError> {
        let pubkey =
            secret_cosmwasm_crypto::secp256k1_recover_pubkey(message_hash, signature, recovery_param)?;
        Ok(pubkey.to_vec())
    }

    fn ed25519_verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Ok(secret_cosmwasm_crypto::ed25519_verify(
            message, signature, public_key,
        )?)
    }

    fn ed25519_batch_verify(
        &self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<bool, VerificationError> {
        Ok(secret_cosmwasm_crypto::ed25519_batch_verify(
            messages,
            signatures,
            public_keys,
        )?)
    }

    fn debug(&self, message: &str) {
        println!("{}", message);
    }

    fn secp256k1_sign(&self, message: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SigningError> {
        Ok(secret_cosmwasm_crypto::secp256k1_sign(message, private_key)?)
    }

    fn ed25519_sign(&self, message: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SigningError> {
        Ok(secret_cosmwasm_crypto::ed25519_sign(message, private_key)?)
    }

    fn check_gas(&self) -> StdResult<u64> {
        Ok(0)
    }

    fn gas_evaporate(&self, _evaporate: u32) -> StdResult<()> {
        Ok(())
    }
}

/// Returns a default enviroment with height, time, chain_id, and contract address
/// You can submit as is to most contracts, or modify height/time if you want to
/// test for expiration.
///
/// This is intended for use in test code only.
pub fn mock_env() -> Env {
    Env {
        block: BlockInfo {
            height: 12_345,
            time: Timestamp::from_nanos(1_571_797_419_879_305_533),
            chain_id: "cosmos-testnet-14002".to_string(),
            #[cfg(feature = "random")]
            random: Some(
                Binary::from_base64("wLsKdf/sYqvSMI0G0aWRjob25mrIB0VQVjTjDXnDafk=").unwrap(),
            ),
        },
        transaction: Some(TransactionInfo { index: 3, hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string() }),
        contract: ContractInfo {
            address: Addr::unchecked(MOCK_CONTRACT_ADDR),
            code_hash: "".to_string(),
        },
    }
}

/// Just set sender and funds for the message.
/// This is intended for use in test code only.
pub fn mock_info(sender: &str, funds: &[Coin]) -> MessageInfo {
    MessageInfo {
        sender: Addr::unchecked(sender),
        funds: funds.to_vec(),
    }
}

/// Creates an IbcChannel for testing. You set a few key parameters for handshaking,
/// If you want to set more, use this as a default and mutate other fields
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel(my_channel_id: &str, order: IbcOrder, version: &str) -> IbcChannel {
    IbcChannel {
        endpoint: IbcEndpoint {
            port_id: "my_port".to_string(),
            channel_id: my_channel_id.to_string(),
        },
        counterparty_endpoint: IbcEndpoint {
            port_id: "their_port".to_string(),
            channel_id: "channel-7".to_string(),
        },
        order,
        version: version.to_string(),
        connection_id: "connection-2".to_string(),
    }
}

/// Creates a IbcChannelOpenMsg::OpenInit for testing ibc_channel_open.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_open_init(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelOpenMsg {
    IbcChannelOpenMsg::new_init(mock_ibc_channel(my_channel_id, order, version))
}

/// Creates a IbcChannelOpenMsg::OpenTry for testing ibc_channel_open.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_open_try(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelOpenMsg {
    IbcChannelOpenMsg::new_try(mock_ibc_channel(my_channel_id, order, version), version)
}

/// Creates a IbcChannelConnectMsg::ConnectAck for testing ibc_channel_connect.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_connect_ack(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelConnectMsg {
    IbcChannelConnectMsg::new_ack(mock_ibc_channel(my_channel_id, order, version), version)
}

/// Creates a IbcChannelConnectMsg::ConnectConfirm for testing ibc_channel_connect.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_connect_confirm(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelConnectMsg {
    IbcChannelConnectMsg::new_confirm(mock_ibc_channel(my_channel_id, order, version))
}

/// Creates a IbcChannelCloseMsg::CloseInit for testing ibc_channel_close.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_close_init(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelCloseMsg {
    IbcChannelCloseMsg::new_init(mock_ibc_channel(my_channel_id, order, version))
}

/// Creates a IbcChannelCloseMsg::CloseConfirm for testing ibc_channel_close.
#[cfg(feature = "stargate")]
pub fn mock_ibc_channel_close_confirm(
    my_channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelCloseMsg {
    IbcChannelCloseMsg::new_confirm(mock_ibc_channel(my_channel_id, order, version))
}

/// Creates a IbcPacketReceiveMsg for testing ibc_packet_receive. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields
#[cfg(feature = "stargate")]
pub fn mock_ibc_packet_recv(
    my_channel_id: &str,
    data: &impl Serialize,
) -> StdResult<IbcPacketReceiveMsg> {
    Ok(IbcPacketReceiveMsg::new(
        IbcPacket {
            data: to_binary(data)?,
            src: IbcEndpoint {
                port_id: "their-port".to_string(),
                channel_id: "channel-1234".to_string(),
            },
            dest: IbcEndpoint {
                port_id: "our-port".to_string(),
                channel_id: my_channel_id.into(),
            },
            sequence: 27,
            timeout: IbcTimeoutBlock {
                revision: 1,
                height: 12345678,
            }
            .into(),
        },
        #[cfg(feature = "ibc3")]
        Addr::unchecked("relayer"),
    ))
}

/// Creates a IbcPacket for testing ibc_packet_{ack,timeout}. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields.
/// The difference from mock_ibc_packet_recv is if `my_channel_id` is src or dest.
#[cfg(feature = "stargate")]
fn mock_ibc_packet(my_channel_id: &str, data: &impl Serialize) -> StdResult<IbcPacket> {
    Ok(IbcPacket {
        data: to_binary(data)?,
        src: IbcEndpoint {
            port_id: "their-port".to_string(),
            channel_id: my_channel_id.into(),
        },
        dest: IbcEndpoint {
            port_id: "our-port".to_string(),
            channel_id: "channel-1234".to_string(),
        },
        sequence: 29,
        timeout: IbcTimeoutBlock {
            revision: 1,
            height: 432332552,
        }
        .into(),
    })
}

/// Creates a IbcPacketAckMsg for testing ibc_packet_ack. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields.
/// The difference from mock_ibc_packet_recv is if `my_channel_id` is src or dest.
#[cfg(feature = "stargate")]
pub fn mock_ibc_packet_ack(
    my_channel_id: &str,
    data: &impl Serialize,
    ack: IbcAcknowledgement,
) -> StdResult<IbcPacketAckMsg> {
    let packet = mock_ibc_packet(my_channel_id, data)?;

    Ok(IbcPacketAckMsg::new(
        ack,
        packet,
        #[cfg(feature = "ibc3")]
        Addr::unchecked("relayer"),
    ))
}

/// Creates a IbcPacketTimeoutMsg for testing ibc_packet_timeout. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields.
/// The difference from mock_ibc_packet_recv is if `my_channel_id` is src or dest./
#[cfg(feature = "stargate")]
pub fn mock_ibc_packet_timeout(
    my_channel_id: &str,
    data: &impl Serialize,
) -> StdResult<IbcPacketTimeoutMsg> {
    let packet = mock_ibc_packet(my_channel_id, data)?;
    Ok(IbcPacketTimeoutMsg::new(
        packet,
        #[cfg(feature = "ibc3")]
        Addr::unchecked("relayer"),
    ))
}

/// The same type as cosmwasm-std's QuerierResult, but easier to reuse in
/// cosmwasm-vm. It might diverge from QuerierResult at some point.
pub type MockQuerierCustomHandlerResult = SystemResult<ContractResult<Binary>>;

/// MockQuerier holds an immutable table of bank balances
/// and configurable handlers for Wasm queries and custom queries.
pub struct MockQuerier<C: DeserializeOwned = Empty> {
    bank: BankQuerier,
    #[cfg(feature = "staking")]
    staking: StakingQuerier,
    wasm: WasmQuerier,
    /// A handler to handle custom queries. This is set to a dummy handler that
    /// always errors by default. Update it via `with_custom_handler`.
    ///
    /// Use box to avoid the need of another generic type
    custom_handler: Box<dyn for<'a> Fn(&'a C) -> MockQuerierCustomHandlerResult>,
}

impl<C: DeserializeOwned> MockQuerier<C> {
    pub fn new(balances: &[(&str, &[Coin])]) -> Self {
        MockQuerier {
            bank: BankQuerier::new(balances),
            #[cfg(feature = "staking")]
            staking: StakingQuerier::default(),
            wasm: WasmQuerier::default(),
            // strange argument notation suggested as a workaround here: https://github.com/rust-lang/rust/issues/41078#issuecomment-294296365
            custom_handler: Box::from(|_: &_| -> MockQuerierCustomHandlerResult {
                SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "custom".to_string(),
                })
            }),
        }
    }

    // set a new balance for the given address and return the old balance
    pub fn update_balance(
        &mut self,
        addr: impl Into<String>,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        self.bank.update_balance(addr, balance)
    }

    #[cfg(feature = "staking")]
    pub fn update_staking(
        &mut self,
        denom: &str,
        validators: &[crate::query::Validator],
        delegations: &[crate::query::FullDelegation],
    ) {
        self.staking = StakingQuerier::new(denom, validators, delegations);
    }

    pub fn update_wasm<WH: 'static>(&mut self, handler: WH)
    where
        WH: Fn(&WasmQuery) -> QuerierResult,
    {
        self.wasm.update_handler(handler)
    }

    pub fn with_custom_handler<CH: 'static>(mut self, handler: CH) -> Self
    where
        CH: Fn(&C) -> MockQuerierCustomHandlerResult,
    {
        self.custom_handler = Box::from(handler);
        self
    }
}

impl Default for MockQuerier {
    fn default() -> Self {
        MockQuerier::new(&[])
    }
}

impl<C: CustomQuery + DeserializeOwned> Querier for MockQuerier<C> {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<C> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl<C: CustomQuery + DeserializeOwned> MockQuerier<C> {
    pub fn handle_query(&self, request: &QueryRequest<C>) -> QuerierResult {
        match &request {
            QueryRequest::Bank(bank_query) => self.bank.query(bank_query),
            QueryRequest::Custom(custom_query) => (*self.custom_handler)(custom_query),
            #[cfg(feature = "staking")]
            QueryRequest::Staking(staking_query) => self.staking.query(staking_query),
            QueryRequest::Wasm(msg) => self.wasm.query(msg),
            #[cfg(feature = "stargate")]
            QueryRequest::Stargate { .. } => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "Stargate".to_string(),
            }),
            #[cfg(feature = "stargate")]
            QueryRequest::Ibc(_) => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "Ibc".to_string(),
            }),
        }
    }
}

struct WasmQuerier {
    /// A handler to handle Wasm queries. This is set to a dummy handler that
    /// always errors by default. Update it via `with_custom_handler`.
    ///
    /// Use box to avoid the need of generic type.
    handler: Box<dyn for<'a> Fn(&'a WasmQuery) -> QuerierResult>,
}

impl WasmQuerier {
    fn new(handler: Box<dyn for<'a> Fn(&'a WasmQuery) -> QuerierResult>) -> Self {
        Self { handler }
    }

    fn update_handler<WH: 'static>(&mut self, handler: WH)
    where
        WH: Fn(&WasmQuery) -> QuerierResult,
    {
        self.handler = Box::from(handler)
    }

    fn query(&self, request: &WasmQuery) -> QuerierResult {
        (*self.handler)(request)
    }
}

impl Default for WasmQuerier {
    fn default() -> Self {
        let handler = Box::from(|request: &WasmQuery| -> QuerierResult {
            let addr = match request {
                WasmQuery::Smart { contract_addr, .. } => contract_addr,
                WasmQuery::ContractInfo { contract_addr, .. } => contract_addr,
                WasmQuery::Raw { .. } => {
                    return SystemResult::Err(SystemError::InvalidRequest {
                        error: "raw queries are unsupported".to_string(),
                        request: Default::default(),
                    })
                }
            }
            .clone();
            SystemResult::Err(SystemError::NoSuchContract { addr })
        });
        Self::new(handler)
    }
}

#[derive(Clone, Default)]
pub struct BankQuerier {
    #[allow(dead_code)]
    /// HashMap<denom, amount>
    supplies: HashMap<String, Uint128>,
    /// HashMap<address, coins>
    balances: HashMap<String, Vec<Coin>>,
}

impl BankQuerier {
    pub fn new(balances: &[(&str, &[Coin])]) -> Self {
        let balances: HashMap<_, _> = balances
            .iter()
            .map(|(s, c)| (s.to_string(), c.to_vec()))
            .collect();

        BankQuerier {
            supplies: Self::calculate_supplies(&balances),
            balances,
        }
    }

    pub fn update_balance(
        &mut self,
        addr: impl Into<String>,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        let result = self.balances.insert(addr.into(), balance);
        self.supplies = Self::calculate_supplies(&self.balances);

        result
    }

    fn calculate_supplies(balances: &HashMap<String, Vec<Coin>>) -> HashMap<String, Uint128> {
        let mut supplies = HashMap::new();

        let all_coins = balances.iter().flat_map(|(_, coins)| coins);

        for coin in all_coins {
            *supplies
                .entry(coin.denom.clone())
                .or_insert_with(Uint128::zero) += coin.amount;
        }

        supplies
    }

    pub fn query(&self, request: &BankQuery) -> QuerierResult {
        let contract_result: ContractResult<Binary> = match request {
            #[cfg(feature = "cosmwasm_1_1")]
            BankQuery::Supply { denom } => {
                let amount = self
                    .supplies
                    .get(denom)
                    .cloned()
                    .unwrap_or_else(Uint128::zero);
                let bank_res = SupplyResponse {
                    amount: Coin {
                        amount,
                        denom: denom.to_string(),
                    },
                };
                to_binary(&bank_res).into()
            }
            BankQuery::Balance { address, denom } => {
                // proper error on not found, serialize result on found
                let amount = self
                    .balances
                    .get(address)
                    .and_then(|v| v.iter().find(|c| &c.denom == denom).map(|c| c.amount))
                    .unwrap_or_default();
                let bank_res = BalanceResponse {
                    amount: Coin {
                        amount,
                        denom: denom.to_string(),
                    },
                };
                to_binary(&bank_res).into()
            }
            BankQuery::AllBalances { address } => {
                // proper error on not found, serialize result on found
                let bank_res = AllBalanceResponse {
                    amount: self.balances.get(address).cloned().unwrap_or_default(),
                };
                to_binary(&bank_res).into()
            }
        };
        // system result is always ok in the mock implementation
        SystemResult::Ok(contract_result)
    }
}

#[cfg(feature = "staking")]
#[derive(Clone, Default)]
pub struct StakingQuerier {
    denom: String,
    validators: Vec<Validator>,
    delegations: Vec<FullDelegation>,
}

#[cfg(feature = "staking")]
impl StakingQuerier {
    pub fn new(denom: &str, validators: &[Validator], delegations: &[FullDelegation]) -> Self {
        StakingQuerier {
            denom: denom.to_string(),
            validators: validators.to_vec(),
            delegations: delegations.to_vec(),
        }
    }

    pub fn query(&self, request: &StakingQuery) -> QuerierResult {
        let contract_result: ContractResult<Binary> = match request {
            StakingQuery::BondedDenom {} => {
                let res = BondedDenomResponse {
                    denom: self.denom.clone(),
                };
                to_binary(&res).into()
            }
            StakingQuery::AllValidators {} => {
                let res = AllValidatorsResponse {
                    validators: self.validators.clone(),
                };
                to_binary(&res).into()
            }
            StakingQuery::Validator { address } => {
                let validator: Option<Validator> = self
                    .validators
                    .iter()
                    .find(|validator| validator.address == *address)
                    .cloned();
                let res = ValidatorResponse { validator };
                to_binary(&res).into()
            }
            StakingQuery::AllDelegations { delegator } => {
                let delegations: Vec<_> = self
                    .delegations
                    .iter()
                    .filter(|d| d.delegator.as_str() == delegator)
                    .cloned()
                    .map(|d| d.into())
                    .collect();
                let res = AllDelegationsResponse { delegations };
                to_binary(&res).into()
            }
            StakingQuery::Delegation {
                delegator,
                validator,
            } => {
                let delegation = self
                    .delegations
                    .iter()
                    .find(|d| d.delegator.as_str() == delegator && d.validator == *validator);
                let res = DelegationResponse {
                    delegation: delegation.cloned(),
                };
                to_binary(&res).into()
            }
        };
        // system result is always ok in the mock implementation
        SystemResult::Ok(contract_result)
    }
}

/// Performs a perfect shuffle (in shuffle)
///
/// https://en.wikipedia.org/wiki/Riffle_shuffle_permutation#Perfect_shuffles
/// https://en.wikipedia.org/wiki/In_shuffle
///
/// The number of shuffles required to restore the original order are listed in
/// https://oeis.org/A002326, e.g.:
///
/// ```ignore
/// 2: 2
/// 4: 4
/// 6: 3
/// 8: 6
/// 10: 10
/// 12: 12
/// 14: 4
/// 16: 8
/// 18: 18
/// 20: 6
/// 22: 11
/// 24: 20
/// 26: 18
/// 28: 28
/// 30: 5
/// 32: 10
/// 34: 12
/// 36: 36
/// 38: 12
/// 40: 20
/// 42: 14
/// 44: 12
/// 46: 23
/// 48: 21
/// 50: 8
/// 52: 52
/// 54: 20
/// 56: 18
/// 58: 58
/// 60: 60
/// 62: 6
/// 64: 12
/// 66: 66
/// 68: 22
/// 70: 35
/// 72: 9
/// 74: 20
/// ```
pub fn riffle_shuffle<T: Clone>(input: &[T]) -> Vec<T> {
    assert!(
        input.len() % 2 == 0,
        "Method only defined for even number of elements"
    );
    let mid = input.len() / 2;
    let (left, right) = input.split_at(mid);
    let mut out = Vec::<T>::with_capacity(input.len());
    for i in 0..mid {
        out.push(right[i].clone());
        out.push(left[i].clone());
    }
    out
}

pub fn digit_sum(input: &[u8]) -> usize {
    input.iter().fold(0, |sum, val| sum + (*val as usize))
}

/// Only for test code. This bypasses assertions in new, allowing us to create _*
/// Attributes to simulate responses from the blockchain
pub fn mock_wasmd_attr(key: impl Into<String>, value: impl Into<String>) -> Attribute {
    Attribute {
        key: key.into(),
        value: value.into(),
        encrypted: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coin, coins, from_binary, to_binary, ContractInfoResponse, Response};
    #[cfg(feature = "staking")]
    use crate::{Decimal, Delegation};
    use hex_literal::hex;
    use serde::Deserialize;

    const SECP256K1_MSG_HASH_HEX: &str =
        "5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0";
    const SECP256K1_SIG_HEX: &str = "207082eb2c3dfa0b454e0906051270ba4074ac93760ba9e7110cd9471475111151eb0dbbc9920e72146fb564f99d039802bf6ef2561446eb126ef364d21ee9c4";
    const SECP256K1_PUBKEY_HEX: &str = "04051c1ee2190ecfb174bfe4f90763f2b4ff7517b70a2aec1876ebcfd644c4633fb03f3cfbd94b1f376e34592d9d41ccaf640bb751b00a1fadeb0c01157769eb73";

    const ED25519_MSG_HEX: &str = "72";
    const ED25519_SIG_HEX: &str = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00";
    const ED25519_PUBKEY_HEX: &str =
        "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";

    #[test]
    fn mock_info_works() {
        let info = mock_info("my name", &coins(100, "atom"));
        assert_eq!(
            info,
            MessageInfo {
                sender: Addr::unchecked("my name"),
                funds: vec![Coin {
                    amount: 100u128.into(),
                    denom: "atom".into(),
                }]
            }
        );
    }

    #[test]
    fn addr_validate_works() {
        let api = MockApi::default();

        // valid
        let addr = api.addr_validate("foobar123").unwrap();
        assert_eq!(addr, "foobar123");

        // invalid: too short
        api.addr_validate("").unwrap_err();
        // invalid: not normalized
        api.addr_validate("Foobar123").unwrap_err();
        api.addr_validate("FOOBAR123").unwrap_err();
    }

    #[test]
    fn addr_canonicalize_works() {
        let api = MockApi::default();

        api.addr_canonicalize("foobar123").unwrap();

        // is case insensitive
        let data1 = api.addr_canonicalize("foo123").unwrap();
        let data2 = api.addr_canonicalize("FOO123").unwrap();
        assert_eq!(data1, data2);
    }

    #[test]
    fn canonicalize_and_humanize_restores_original() {
        let api = MockApi::default();

        // simple
        let original = String::from("shorty");
        let canonical = api.addr_canonicalize(&original).unwrap();
        let recovered = api.addr_humanize(&canonical).unwrap();
        assert_eq!(recovered, original);

        // normalizes input
        let original = String::from("CosmWasmChef");
        let canonical = api.addr_canonicalize(&original).unwrap();
        let recovered = api.addr_humanize(&canonical).unwrap();
        assert_eq!(recovered, "cosmwasmchef");
    }

    #[test]
    #[should_panic(expected = "address too short")]
    fn addr_canonicalize_min_input_length() {
        let api = MockApi::default();
        let human = String::from("1");
        let _ = api.addr_canonicalize(&human).unwrap();
    }

    #[test]
    #[should_panic(expected = "address too long")]
    fn addr_canonicalize_max_input_length() {
        let api = MockApi::default();
        let human =
            String::from("some-extremely-long-address-not-supported-by-this-api-longer-than-54");
        let _ = api.addr_canonicalize(&human).unwrap();
    }

    #[test]
    #[should_panic(expected = "length not correct")]
    fn addr_humanize_input_length() {
        let api = MockApi::default();
        let input = CanonicalAddr::from(vec![61; 11]);
        api.addr_humanize(&input).unwrap();
    }

    // Basic "works" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn secp256k1_verify_works() {
        let api = MockApi::default();

        let hash = hex::decode(SECP256K1_MSG_HASH_HEX).unwrap();
        let signature = hex::decode(SECP256K1_SIG_HEX).unwrap();
        let public_key = hex::decode(SECP256K1_PUBKEY_HEX).unwrap();

        assert!(api
            .secp256k1_verify(&hash, &signature, &public_key)
            .unwrap());
    }

    // Basic "fails" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn secp256k1_verify_fails() {
        let api = MockApi::default();

        let mut hash = hex::decode(SECP256K1_MSG_HASH_HEX).unwrap();
        // alter hash
        hash[0] ^= 0x01;
        let signature = hex::decode(SECP256K1_SIG_HEX).unwrap();
        let public_key = hex::decode(SECP256K1_PUBKEY_HEX).unwrap();

        assert!(!api
            .secp256k1_verify(&hash, &signature, &public_key)
            .unwrap());
    }

    // Basic "errors" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn secp256k1_verify_errs() {
        let api = MockApi::default();

        let hash = hex::decode(SECP256K1_MSG_HASH_HEX).unwrap();
        let signature = hex::decode(SECP256K1_SIG_HEX).unwrap();
        let public_key = vec![];

        let res = api.secp256k1_verify(&hash, &signature, &public_key);
        assert_eq!(res.unwrap_err(), VerificationError::InvalidPubkeyFormat);
    }

    #[test]
    fn secp256k1_recover_pubkey_works() {
        let api = MockApi::default();

        // https://gist.github.com/webmaster128/130b628d83621a33579751846699ed15
        let hash = hex!("5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0");
        let signature = hex!("45c0b7f8c09a9e1f1cea0c25785594427b6bf8f9f878a8af0b1abbb48e16d0920d8becd0c220f67c51217eecfd7184ef0732481c843857e6bc7fc095c4f6b788");
        let recovery_param = 1;
        let expected = hex!("044a071e8a6e10aada2b8cf39fa3b5fb3400b04e99ea8ae64ceea1a977dbeaf5d5f8c8fbd10b71ab14cd561f7df8eb6da50f8a8d81ba564342244d26d1d4211595");

        let pubkey = api
            .secp256k1_recover_pubkey(&hash, &signature, recovery_param)
            .unwrap();
        assert_eq!(pubkey, expected);
    }

    #[test]
    fn secp256k1_recover_pubkey_fails_for_wrong_recovery_param() {
        let api = MockApi::default();

        // https://gist.github.com/webmaster128/130b628d83621a33579751846699ed15
        let hash = hex!("5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0");
        let signature = hex!("45c0b7f8c09a9e1f1cea0c25785594427b6bf8f9f878a8af0b1abbb48e16d0920d8becd0c220f67c51217eecfd7184ef0732481c843857e6bc7fc095c4f6b788");
        let _recovery_param = 1;
        let expected = hex!("044a071e8a6e10aada2b8cf39fa3b5fb3400b04e99ea8ae64ceea1a977dbeaf5d5f8c8fbd10b71ab14cd561f7df8eb6da50f8a8d81ba564342244d26d1d4211595");

        // Wrong recovery param leads to different pubkey
        let pubkey = api.secp256k1_recover_pubkey(&hash, &signature, 0).unwrap();
        assert_eq!(pubkey.len(), 65);
        assert_ne!(pubkey, expected);

        // Invalid recovery param leads to error
        let result = api.secp256k1_recover_pubkey(&hash, &signature, 42);
        match result.unwrap_err() {
            RecoverPubkeyError::InvalidRecoveryParam => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn secp256k1_recover_pubkey_fails_for_wrong_hash() {
        let api = MockApi::default();

        // https://gist.github.com/webmaster128/130b628d83621a33579751846699ed15
        let hash = hex!("5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0");
        let signature = hex!("45c0b7f8c09a9e1f1cea0c25785594427b6bf8f9f878a8af0b1abbb48e16d0920d8becd0c220f67c51217eecfd7184ef0732481c843857e6bc7fc095c4f6b788");
        let recovery_param = 1;
        let expected = hex!("044a071e8a6e10aada2b8cf39fa3b5fb3400b04e99ea8ae64ceea1a977dbeaf5d5f8c8fbd10b71ab14cd561f7df8eb6da50f8a8d81ba564342244d26d1d4211595");

        // Wrong hash
        let mut corrupted_hash = hash;
        corrupted_hash[0] ^= 0x01;
        let pubkey = api
            .secp256k1_recover_pubkey(&corrupted_hash, &signature, recovery_param)
            .unwrap();
        assert_eq!(pubkey.len(), 65);
        assert_ne!(pubkey, expected);

        // Malformed hash
        let mut malformed_hash = hash.to_vec();
        malformed_hash.push(0x8a);
        let result = api.secp256k1_recover_pubkey(&malformed_hash, &signature, recovery_param);
        match result.unwrap_err() {
            RecoverPubkeyError::InvalidHashFormat => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    // Basic "works" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn ed25519_verify_works() {
        let api = MockApi::default();

        let msg = hex::decode(ED25519_MSG_HEX).unwrap();
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key = hex::decode(ED25519_PUBKEY_HEX).unwrap();

        assert!(api.ed25519_verify(&msg, &signature, &public_key).unwrap());
    }

    // Basic "fails" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn ed25519_verify_fails() {
        let api = MockApi::default();

        let mut msg = hex::decode(ED25519_MSG_HEX).unwrap();
        // alter msg
        msg[0] ^= 0x01;
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key = hex::decode(ED25519_PUBKEY_HEX).unwrap();

        assert!(!api.ed25519_verify(&msg, &signature, &public_key).unwrap());
    }

    // Basic "errors" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn ed25519_verify_errs() {
        let api = MockApi::default();

        let msg = hex::decode(ED25519_MSG_HEX).unwrap();
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key = vec![];

        let res = api.ed25519_verify(&msg, &signature, &public_key);
        assert_eq!(res.unwrap_err(), VerificationError::InvalidPubkeyFormat);
    }

    // Basic "works" test.
    #[test]
    fn ed25519_batch_verify_works() {
        let api = MockApi::default();

        let msg = hex::decode(ED25519_MSG_HEX).unwrap();
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key = hex::decode(ED25519_PUBKEY_HEX).unwrap();

        let msgs: Vec<&[u8]> = vec![&msg];
        let signatures: Vec<&[u8]> = vec![&signature];
        let public_keys: Vec<&[u8]> = vec![&public_key];

        assert!(api
            .ed25519_batch_verify(&msgs, &signatures, &public_keys)
            .unwrap());
    }

    // Basic "fails" test.
    #[test]
    fn ed25519_batch_verify_fails() {
        let api = MockApi::default();

        let mut msg = hex::decode(ED25519_MSG_HEX).unwrap();
        // alter msg
        msg[0] ^= 0x01;
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key = hex::decode(ED25519_PUBKEY_HEX).unwrap();

        let msgs: Vec<&[u8]> = vec![&msg];
        let signatures: Vec<&[u8]> = vec![&signature];
        let public_keys: Vec<&[u8]> = vec![&public_key];

        assert!(!api
            .ed25519_batch_verify(&msgs, &signatures, &public_keys)
            .unwrap());
    }

    // Basic "errors" test.
    #[test]
    fn ed25519_batch_verify_errs() {
        let api = MockApi::default();

        let msg = hex::decode(ED25519_MSG_HEX).unwrap();
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key: Vec<u8> = vec![0u8; 0];

        let msgs: Vec<&[u8]> = vec![msg.as_slice()];
        let signatures: Vec<&[u8]> = vec![signature.as_slice()];
        let public_keys: Vec<&[u8]> = vec![&public_key];

        let res = api.ed25519_batch_verify(&msgs, &signatures, &public_keys);
        assert_eq!(res.unwrap_err(), VerificationError::InvalidPubkeyFormat);
    }

    #[cfg(feature = "cosmwasm_1_1")]
    #[test]
    fn bank_querier_supply() {
        let addr1 = String::from("foo");
        let balance1 = vec![coin(123, "ELF"), coin(777, "FLY")];

        let addr2 = String::from("bar");
        let balance2 = coins(321, "ELF");

        let bank = BankQuerier::new(&[(&addr1, &balance1), (&addr2, &balance2)]);

        let elf = bank
            .query(&BankQuery::Supply {
                denom: "ELF".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: SupplyResponse = from_binary(&elf).unwrap();
        assert_eq!(res.amount, coin(444, "ELF"));

        let fly = bank
            .query(&BankQuery::Supply {
                denom: "FLY".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: SupplyResponse = from_binary(&fly).unwrap();
        assert_eq!(res.amount, coin(777, "FLY"));

        // if a denom does not exist, should return zero amount, instead of throwing an error
        let atom = bank
            .query(&BankQuery::Supply {
                denom: "ATOM".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: SupplyResponse = from_binary(&atom).unwrap();
        assert_eq!(res.amount, coin(0, "ATOM"));
    }

    #[test]
    fn bank_querier_all_balances() {
        let addr = String::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let bank = BankQuerier::new(&[(&addr, &balance)]);

        let all = bank
            .query(&BankQuery::AllBalances { address: addr })
            .unwrap()
            .unwrap();
        let res: AllBalanceResponse = from_binary(&all).unwrap();
        assert_eq!(&res.amount, &balance);
    }

    #[test]
    fn bank_querier_one_balance() {
        let addr = String::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let bank = BankQuerier::new(&[(&addr, &balance)]);

        // one match
        let fly = bank
            .query(&BankQuery::Balance {
                address: addr.clone(),
                denom: "FLY".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_binary(&fly).unwrap();
        assert_eq!(res.amount, coin(777, "FLY"));

        // missing denom
        let miss = bank
            .query(&BankQuery::Balance {
                address: addr,
                denom: "MISS".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_binary(&miss).unwrap();
        assert_eq!(res.amount, coin(0, "MISS"));
    }

    #[test]
    fn bank_querier_missing_account() {
        let addr = String::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let bank = BankQuerier::new(&[(&addr, &balance)]);

        // all balances on empty account is empty vec
        let all = bank
            .query(&BankQuery::AllBalances {
                address: String::from("elsewhere"),
            })
            .unwrap()
            .unwrap();
        let res: AllBalanceResponse = from_binary(&all).unwrap();
        assert_eq!(res.amount, vec![]);

        // any denom on balances on empty account is empty coin
        let miss = bank
            .query(&BankQuery::Balance {
                address: String::from("elsewhere"),
                denom: "ELF".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_binary(&miss).unwrap();
        assert_eq!(res.amount, coin(0, "ELF"));
    }

    #[cfg(feature = "staking")]
    #[test]
    fn staking_querier_all_validators() {
        let val1 = Validator {
            address: String::from("validator-one"),
            commission: Decimal::percent(1),
            max_commission: Decimal::percent(3),
            max_change_rate: Decimal::percent(1),
        };
        let val2 = Validator {
            address: String::from("validator-two"),
            commission: Decimal::permille(15),
            max_commission: Decimal::permille(40),
            max_change_rate: Decimal::permille(5),
        };

        let staking = StakingQuerier::new("ustake", &[val1.clone(), val2.clone()], &[]);

        // one match
        let raw = staking
            .query(&StakingQuery::AllValidators {})
            .unwrap()
            .unwrap();
        let vals: AllValidatorsResponse = from_binary(&raw).unwrap();
        assert_eq!(vals.validators, vec![val1, val2]);
    }

    #[cfg(feature = "staking")]
    #[test]
    fn staking_querier_validator() {
        let address1 = String::from("validator-one");
        let address2 = String::from("validator-two");
        let address_non_existent = String::from("wannabe-validator");

        let val1 = Validator {
            address: address1.clone(),
            commission: Decimal::percent(1),
            max_commission: Decimal::percent(3),
            max_change_rate: Decimal::percent(1),
        };
        let val2 = Validator {
            address: address2.clone(),
            commission: Decimal::permille(15),
            max_commission: Decimal::permille(40),
            max_change_rate: Decimal::permille(5),
        };

        let staking = StakingQuerier::new("ustake", &[val1.clone(), val2.clone()], &[]);

        // query 1
        let raw = staking
            .query(&StakingQuery::Validator { address: address1 })
            .unwrap()
            .unwrap();
        let res: ValidatorResponse = from_binary(&raw).unwrap();
        assert_eq!(res.validator, Some(val1));

        // query 2
        let raw = staking
            .query(&StakingQuery::Validator { address: address2 })
            .unwrap()
            .unwrap();
        let res: ValidatorResponse = from_binary(&raw).unwrap();
        assert_eq!(res.validator, Some(val2));

        // query non-existent
        let raw = staking
            .query(&StakingQuery::Validator {
                address: address_non_existent,
            })
            .unwrap()
            .unwrap();
        let res: ValidatorResponse = from_binary(&raw).unwrap();
        assert_eq!(res.validator, None);
    }

    #[cfg(feature = "staking")]
    // gets delegators from query or panic
    fn get_all_delegators(
        staking: &StakingQuerier,
        delegator: impl Into<String>,
    ) -> Vec<Delegation> {
        let raw = staking
            .query(&StakingQuery::AllDelegations {
                delegator: delegator.into(),
            })
            .unwrap()
            .unwrap();
        let dels: AllDelegationsResponse = from_binary(&raw).unwrap();
        dels.delegations
    }

    #[cfg(feature = "staking")]
    // gets full delegators from query or panic
    fn get_delegator(
        staking: &StakingQuerier,
        delegator: impl Into<String>,
        validator: impl Into<String>,
    ) -> Option<FullDelegation> {
        let raw = staking
            .query(&StakingQuery::Delegation {
                delegator: delegator.into(),
                validator: validator.into(),
            })
            .unwrap()
            .unwrap();
        let dels: DelegationResponse = from_binary(&raw).unwrap();
        dels.delegation
    }

    #[cfg(feature = "staking")]
    #[test]
    fn staking_querier_delegations() {
        let val1 = String::from("validator-one");
        let val2 = String::from("validator-two");

        let user_a = Addr::unchecked("investor");
        let user_b = Addr::unchecked("speculator");
        let user_c = Addr::unchecked("hodler");

        // we need multiple validators per delegator, so the queries provide different results
        let del1a = FullDelegation {
            delegator: user_a.clone(),
            validator: val1.clone(),
            amount: coin(100, "ustake"),
            can_redelegate: coin(100, "ustake"),
            accumulated_rewards: coins(5, "ustake"),
        };
        let del2a = FullDelegation {
            delegator: user_a.clone(),
            validator: val2.clone(),
            amount: coin(500, "ustake"),
            can_redelegate: coin(500, "ustake"),
            accumulated_rewards: coins(20, "ustake"),
        };

        // note we cannot have multiple delegations on one validator, they are collapsed into one
        let del1b = FullDelegation {
            delegator: user_b.clone(),
            validator: val1.clone(),
            amount: coin(500, "ustake"),
            can_redelegate: coin(0, "ustake"),
            accumulated_rewards: coins(0, "ustake"),
        };

        // and another one on val2
        let del2c = FullDelegation {
            delegator: user_c.clone(),
            validator: val2.clone(),
            amount: coin(8888, "ustake"),
            can_redelegate: coin(4567, "ustake"),
            accumulated_rewards: coins(900, "ustake"),
        };

        let staking = StakingQuerier::new(
            "ustake",
            &[],
            &[del1a.clone(), del1b.clone(), del2a.clone(), del2c.clone()],
        );

        // get all for user a
        let dels = get_all_delegators(&staking, user_a.clone());
        assert_eq!(dels, vec![del1a.clone().into(), del2a.clone().into()]);

        // get all for user b
        let dels = get_all_delegators(&staking, user_b.clone());
        assert_eq!(dels, vec![del1b.clone().into()]);

        // get all for user c
        let dels = get_all_delegators(&staking, user_c.clone());
        assert_eq!(dels, vec![del2c.clone().into()]);

        // for user with no delegations...
        let dels = get_all_delegators(&staking, String::from("no one"));
        assert_eq!(dels, vec![]);

        // filter a by validator (1 and 1)
        let dels = get_delegator(&staking, user_a.clone(), val1.clone());
        assert_eq!(dels, Some(del1a));
        let dels = get_delegator(&staking, user_a, val2.clone());
        assert_eq!(dels, Some(del2a));

        // filter b by validator (2 and 0)
        let dels = get_delegator(&staking, user_b.clone(), val1.clone());
        assert_eq!(dels, Some(del1b));
        let dels = get_delegator(&staking, user_b, val2.clone());
        assert_eq!(dels, None);

        // filter c by validator (0 and 1)
        let dels = get_delegator(&staking, user_c.clone(), val1);
        assert_eq!(dels, None);
        let dels = get_delegator(&staking, user_c, val2);
        assert_eq!(dels, Some(del2c));
    }

    #[test]
    fn wasm_querier_works() {
        let mut querier = WasmQuerier::default();

        let any_addr = "foo".to_string();
        let any_code_hash = "goo".to_string();

        // Query WasmQuery::Smart
        let system_err = querier
            .query(&WasmQuery::Smart {
                contract_addr: any_addr.clone(),
                code_hash: any_code_hash.clone(),
                msg: b"{}".into(),
            })
            .unwrap_err();
        match system_err {
            SystemError::NoSuchContract { addr } => assert_eq!(addr, any_addr),
            err => panic!("Unexpected error: {:?}", err),
        }

        // Query WasmQuery::ContractInfo
        let system_err = querier
            .query(&WasmQuery::ContractInfo {
                contract_addr: any_addr.clone(),
            })
            .unwrap_err();
        match system_err {
            SystemError::NoSuchContract { addr } => assert_eq!(addr, any_addr),
            err => panic!("Unexpected error: {:?}", err),
        }

        querier.update_handler(|request| {
            let constract1 = Addr::unchecked("contract1");
            let mut storage1 = HashMap::<Binary, Binary>::default();
            storage1.insert(b"the key".into(), b"the value".into());

            match request {
                WasmQuery::Smart {
                    contract_addr, msg, ..
                } => {
                    if *contract_addr == constract1 {
                        #[derive(Deserialize)]
                        struct MyMsg {}
                        let _msg: MyMsg = match from_binary(msg) {
                            Ok(msg) => msg,
                            Err(err) => {
                                return SystemResult::Ok(ContractResult::Err(err.to_string()))
                            }
                        };
                        let response: Response = Response::new().set_data(b"good");
                        SystemResult::Ok(ContractResult::Ok(to_binary(&response).unwrap()))
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        })
                    }
                }
                WasmQuery::ContractInfo { contract_addr } => {
                    if *contract_addr == constract1 {
                        let response = ContractInfoResponse {
                            code_id: 4,
                            creator: "lalala".into(),
                            pinned: false,
                            ibc_port: None,
                        };
                        SystemResult::Ok(ContractResult::Ok(to_binary(&response).unwrap()))
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        })
                    }
                }
                _ => {
                    panic!("Raw queries are unsupported")
                }
            }
        });

        // WasmQuery::Smart
        let result = querier.query(&WasmQuery::Smart {
            contract_addr: "contract1".into(),
            code_hash: "code_hash1".into(),
            msg: b"{}".into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(
                value,
                br#"{"messages":[],"attributes":[],"events":[],"data":"Z29vZA=="}"# as &[u8]
            ),
            res => panic!("Unexpected result: {:?}", res),
        }
        let result = querier.query(&WasmQuery::Smart {
            contract_addr: "contract1".into(),
            code_hash: "code_hash1".into(),
            msg: b"a broken request".into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Err(err)) => {
                assert_eq!(err, "Error parsing into type secret_cosmwasm_std::testing::mock::tests::wasm_querier_works::{{closure}}::MyMsg: Invalid type")
            }
            res => panic!("Unexpected result: {:?}", res),
        }

        // WasmQuery::ContractInfo
        let result = querier.query(&WasmQuery::ContractInfo {
            contract_addr: "contract1".into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(
                value,
                br#"{"code_id":4,"creator":"lalala","pinned":false,"ibc_port":null}"# as &[u8]
            ),
            res => panic!("Unexpected result: {:?}", res),
        }
    }

    #[test]
    fn riffle_shuffle_works() {
        // Example from https://en.wikipedia.org/wiki/In_shuffle
        let start = [0xA, 0x2, 0x3, 0x4, 0x5, 0x6];
        let round1 = riffle_shuffle(&start);
        assert_eq!(round1, [0x4, 0xA, 0x5, 0x2, 0x6, 0x3]);
        let round2 = riffle_shuffle(&round1);
        assert_eq!(round2, [0x2, 0x4, 0x6, 0xA, 0x3, 0x5]);
        let round3 = riffle_shuffle(&round2);
        assert_eq!(round3, start);

        // For 14 elements, the original order is restored after 4 executions
        // See https://en.wikipedia.org/wiki/In_shuffle#Mathematics and https://oeis.org/A002326
        let original = [12, 33, 76, 576, 0, 44, 1, 14, 78, 99, 871212, -7, 2, -1];
        let mut result = Vec::from(original);
        for _ in 0..4 {
            result = riffle_shuffle(&result);
        }
        assert_eq!(result, original);

        // For 24 elements, the original order is restored after 20 executions
        let original = [
            7, 4, 2, 4656, 23, 45, 23, 1, 12, 76, 576, 0, 12, 1, 14, 78, 99, 12, 1212, 444, 31,
            111, 424, 34,
        ];
        let mut result = Vec::from(original);
        for _ in 0..20 {
            result = riffle_shuffle(&result);
        }
        assert_eq!(result, original);
    }

    #[test]
    fn digit_sum_works() {
        assert_eq!(digit_sum(&[]), 0);
        assert_eq!(digit_sum(&[0]), 0);
        assert_eq!(digit_sum(&[0, 0]), 0);
        assert_eq!(digit_sum(&[0, 0, 0]), 0);

        assert_eq!(digit_sum(&[1, 0, 0]), 1);
        assert_eq!(digit_sum(&[0, 1, 0]), 1);
        assert_eq!(digit_sum(&[0, 0, 1]), 1);

        assert_eq!(digit_sum(&[1, 2, 3]), 6);

        assert_eq!(digit_sum(&[255, 1]), 256);
    }
}
