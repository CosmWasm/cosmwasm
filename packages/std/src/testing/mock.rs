use crate::prelude::*;
use crate::HashFunction;
use crate::{Addr, CanonicalAddr, Timestamp};
use alloc::collections::BTreeMap;
#[cfg(feature = "cosmwasm_1_3")]
use alloc::collections::BTreeSet;
use bech32::primitives::decode::CheckedHrpstring;
use bech32::{encode, Bech32, Hrp};
use core::marker::PhantomData;
#[cfg(feature = "cosmwasm_1_3")]
use core::ops::Bound;
use rand_core::OsRng;
use serde::de::DeserializeOwned;
#[cfg(any(feature = "stargate", feature = "ibc2"))]
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::coin::Coin;
use crate::deps::OwnedDeps;
#[cfg(feature = "stargate")]
use crate::ibc::{
    IbcAcknowledgement, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcEndpoint, IbcOrder, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcTimeoutBlock,
};
#[cfg(feature = "ibc2")]
use crate::ibc2::{
    Ibc2PacketAckMsg, Ibc2PacketReceiveMsg, Ibc2PacketSendMsg, Ibc2PacketTimeoutMsg, Ibc2Payload,
};
#[cfg(feature = "cosmwasm_1_1")]
use crate::query::SupplyResponse;
#[cfg(feature = "staking")]
use crate::query::{
    AllDelegationsResponse, AllValidatorsResponse, BondedDenomResponse, DelegationResponse,
    FullDelegation, StakingQuery, Validator, ValidatorResponse,
};
use crate::query::{BalanceResponse, BankQuery, CustomQuery, QueryRequest, WasmQuery};
#[cfg(feature = "cosmwasm_1_3")]
use crate::query::{DelegatorWithdrawAddressResponse, DistributionQuery};
use crate::results::{ContractResult, Empty, SystemResult};
use crate::traits::{Api, Querier, QuerierResult};
use crate::types::{BlockInfo, ContractInfo, Env, TransactionInfo};
use crate::{from_json, to_json_binary, Binary, Uint256};
#[cfg(feature = "cosmwasm_1_3")]
use crate::{
    query::{AllDenomMetadataResponse, DecCoin, DenomMetadataResponse},
    PageRequest,
};
use crate::{Attribute, DenomMetadata};
#[cfg(feature = "stargate")]
use crate::{ChannelResponse, IbcQuery, PortIdResponse};
#[cfg(feature = "cosmwasm_1_4")]
use crate::{Decimal256, DelegationRewardsResponse, DelegatorValidatorsResponse};
use crate::{RecoverPubkeyError, StdError, StdResult, SystemError, VerificationError};

use super::MockStorage;

pub const MOCK_CONTRACT_ADDR: &str =
    "cosmwasm1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922tscp8avs";

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

/// Default prefix used when creating Bech32 encoded address.
const BECH32_PREFIX: &str = "cosmwasm";

// MockApi zero pads all human addresses to make them fit the canonical_length
// it trims off zeros for the reverse operation.
// not really smart, but allows us to see a difference (and consistent length for canonical addresses)
#[derive(Copy, Clone)]
pub struct MockApi {
    /// Prefix used for creating addresses in Bech32 encoding.
    bech32_prefix: &'static str,
}

impl Default for MockApi {
    fn default() -> Self {
        MockApi {
            bech32_prefix: BECH32_PREFIX,
        }
    }
}

impl Api for MockApi {
    fn addr_validate(&self, input: &str) -> StdResult<Addr> {
        let canonical = self.addr_canonicalize(input)?;
        let normalized = self.addr_humanize(&canonical)?;
        if input != normalized.as_str() {
            return Err(StdError::generic_err(
                "Invalid input: address not normalized",
            ));
        }
        Ok(Addr::unchecked(input))
    }

    fn addr_canonicalize(&self, input: &str) -> StdResult<CanonicalAddr> {
        let hrp_str = CheckedHrpstring::new::<Bech32>(input)
            .map_err(|_| StdError::generic_err("Error decoding bech32"))?;

        if !hrp_str
            .hrp()
            .as_bytes()
            .eq_ignore_ascii_case(self.bech32_prefix.as_bytes())
        {
            return Err(StdError::generic_err("Wrong bech32 prefix"));
        }

        let bytes: Vec<u8> = hrp_str.byte_iter().collect();
        validate_length(&bytes)?;
        Ok(bytes.into())
    }

    fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr> {
        validate_length(canonical.as_ref())?;

        let prefix = Hrp::parse(self.bech32_prefix)
            .map_err(|_| StdError::generic_err("Invalid bech32 prefix"))?;
        encode::<Bech32>(prefix, canonical.as_slice())
            .map(Addr::unchecked)
            .map_err(|_| StdError::generic_err("Bech32 encoding error"))
    }

    fn bls12_381_aggregate_g1(&self, g1s: &[u8]) -> Result<[u8; 48], VerificationError> {
        cosmwasm_crypto::bls12_381_aggregate_g1(g1s).map_err(Into::into)
    }

    fn bls12_381_aggregate_g2(&self, g2s: &[u8]) -> Result<[u8; 96], VerificationError> {
        cosmwasm_crypto::bls12_381_aggregate_g2(g2s).map_err(Into::into)
    }

    fn bls12_381_pairing_equality(
        &self,
        ps: &[u8],
        qs: &[u8],
        r: &[u8],
        s: &[u8],
    ) -> Result<bool, VerificationError> {
        cosmwasm_crypto::bls12_381_pairing_equality(ps, qs, r, s).map_err(Into::into)
    }

    fn bls12_381_hash_to_g1(
        &self,
        hash_function: HashFunction,
        msg: &[u8],
        dst: &[u8],
    ) -> Result<[u8; 48], VerificationError> {
        Ok(cosmwasm_crypto::bls12_381_hash_to_g1(
            hash_function.into(),
            msg,
            dst,
        ))
    }

    fn bls12_381_hash_to_g2(
        &self,
        hash_function: HashFunction,
        msg: &[u8],
        dst: &[u8],
    ) -> Result<[u8; 96], VerificationError> {
        Ok(cosmwasm_crypto::bls12_381_hash_to_g2(
            hash_function.into(),
            msg,
            dst,
        ))
    }

    fn secp256k1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Ok(cosmwasm_crypto::secp256k1_verify(
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
            cosmwasm_crypto::secp256k1_recover_pubkey(message_hash, signature, recovery_param)?;
        Ok(pubkey.to_vec())
    }

    fn secp256r1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Ok(cosmwasm_crypto::secp256r1_verify(
            message_hash,
            signature,
            public_key,
        )?)
    }

    fn secp256r1_recover_pubkey(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        recovery_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError> {
        let pubkey =
            cosmwasm_crypto::secp256r1_recover_pubkey(message_hash, signature, recovery_param)?;
        Ok(pubkey.to_vec())
    }

    fn ed25519_verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Ok(cosmwasm_crypto::ed25519_verify(
            message, signature, public_key,
        )?)
    }

    fn ed25519_batch_verify(
        &self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<bool, VerificationError> {
        Ok(cosmwasm_crypto::ed25519_batch_verify(
            &mut OsRng,
            messages,
            signatures,
            public_keys,
        )?)
    }

    fn debug(&self, #[allow(unused)] message: &str) {
        println!("{message}");
    }
}

impl MockApi {
    /// Returns [MockApi] with Bech32 prefix set to provided value.
    ///
    /// Bech32 prefix must not be empty.
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::Addr;
    /// # use cosmwasm_std::testing::MockApi;
    /// #
    /// let mock_api = MockApi::default().with_prefix("juno");
    /// let addr = mock_api.addr_make("creator");
    ///
    /// assert_eq!(addr.to_string(), "juno1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqsksmtyp");
    /// ```
    pub fn with_prefix(mut self, prefix: &'static str) -> Self {
        self.bech32_prefix = prefix;
        self
    }

    /// Returns an address built from provided input string.
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::Addr;
    /// # use cosmwasm_std::testing::MockApi;
    /// #
    /// let mock_api = MockApi::default();
    /// let addr = mock_api.addr_make("creator");
    ///
    /// assert_eq!(addr.to_string(), "cosmwasm1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqs8s7vcp");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics when generating a valid address is not possible,
    /// especially when Bech32 prefix set in function [with_prefix](Self::with_prefix) is empty.
    ///
    pub fn addr_make(&self, input: &str) -> Addr {
        let digest = Sha256::digest(input);

        let prefix = match Hrp::parse(self.bech32_prefix) {
            Ok(prefix) => prefix,
            Err(reason) => panic!("Generating address failed with reason: {reason}"),
        };

        match encode::<Bech32>(prefix, &digest) {
            Ok(address) => Addr::unchecked(address),
            Err(reason) => panic!("Generating address failed with reason: {reason}"),
        }
    }
}

/// Does basic validation of the number of bytes in a canonical address
fn validate_length(bytes: &[u8]) -> StdResult<()> {
    match bytes.len() {
        1..=255 => Ok(()),
        _ => Err(StdError::generic_err("Invalid canonical address length")),
    }
}

/// Returns a default environment with height, time, chain_id, and contract address.
/// You can submit as is to most contracts, or modify height/time if you want to
/// test for expiration.
///
/// This is intended for use in test code only.
///
/// The contract address uses the same bech32 prefix as [`MockApi`](crate::testing::MockApi). While
/// this is good for the majority of users, you might need to create your `Env`s
/// differently if you need a valid address using a different prefix.
///
/// ## Examples
///
/// Create an env:
///
/// ```
/// # use cosmwasm_std::{Addr, Binary, BlockInfo, ContractInfo, Env, Timestamp, TransactionInfo};
/// use cosmwasm_std::testing::mock_env;
///
/// let env = mock_env();
/// assert_eq!(env, Env {
///     block: BlockInfo {
///         height: 12_345,
///         time: Timestamp::from_nanos(1_571_797_419_879_305_533),
///         chain_id: "cosmos-testnet-14002".to_string(),
///     },
///     transaction: Some(TransactionInfo::new(3, Binary::from_hex("E5469DACEC17CEF8A260FD37675ED87E7FB6A2B5AD95193C51308006C7E494B3").unwrap())),
///     contract: ContractInfo {
///         address: Addr::unchecked("cosmwasm1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922tscp8avs"),
///     },
/// });
/// ```
///
/// Mutate and reuse environment:
///
/// ```
/// # use cosmwasm_std::{Addr, BlockInfo, ContractInfo, Env, Timestamp, TransactionInfo};
/// use cosmwasm_std::testing::mock_env;
///
/// let env1 = mock_env();
///
/// // First test with `env1`
///
/// let mut env2 = env1.clone();
/// env2.block.height += 1;
/// env2.block.time = env1.block.time.plus_seconds(6);
///
/// // `env2` is one block and 6 seconds later
///
/// let mut env3 = env2.clone();
/// env3.block.height += 1;
/// env3.block.time = env2.block.time.plus_nanos(5_500_000_000);
///
/// // `env3` is one block and 5.5 seconds later
/// ```
pub fn mock_env() -> Env {
    let mut envs = Envs::new(BECH32_PREFIX);
    envs.make()
}

/// A factory type that stores chain information such as bech32 prefix and can make mock `Env`s from there.
///
/// It increments height for each mock call and block time by 5 seconds but is otherwise dumb.
///
/// In contrast to using `mock_env`, the bech32 prefix must always be specified.
///
/// ## Examples
///
/// Typical usage
///
/// ```
/// # use cosmwasm_std::Timestamp;
/// use cosmwasm_std::testing::Envs;
///
/// let mut envs = Envs::new("food");
///
/// let env = envs.make();
/// assert_eq!(env.contract.address.as_str(), "food1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922ts74yrjj");
/// assert_eq!(env.block.height, 12_345);
/// assert_eq!(env.block.time, Timestamp::from_nanos(1_571_797_419_879_305_533));
///
/// let env = envs.make();
/// assert_eq!(env.contract.address.as_str(), "food1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922ts74yrjj");
/// assert_eq!(env.block.height, 12_346);
/// assert_eq!(env.block.time, Timestamp::from_nanos(1_571_797_424_879_305_533));
///
/// let env = envs.make();
/// assert_eq!(env.contract.address.as_str(), "food1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922ts74yrjj");
/// assert_eq!(env.block.height, 12_347);
/// assert_eq!(env.block.time, Timestamp::from_nanos(1_571_797_429_879_305_533));
/// ```
///
/// Or use with iterator
///
/// ```
/// # use cosmwasm_std::Timestamp;
/// use cosmwasm_std::testing::Envs;
///
/// let mut envs = Envs::new("food");
///
/// for (index, env) in envs.take(100).enumerate() {
///     assert_eq!(env.contract.address.as_str(), "food1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922ts74yrjj");
///     assert_eq!(env.block.height, 12_345 + index as u64);
///     assert_eq!(env.block.time, Timestamp::from_nanos(1_571_797_419_879_305_533).plus_seconds((index*5) as u64));
/// }
/// ```
pub struct Envs {
    chain_id: String,
    contract_address: Addr,
    /// The number of nanoseconds between two consecutive blocks
    block_time: u64,
    last_height: u64,
    last_time: Timestamp,
}

/// Options to create an `Envs` instance.
///
/// ## Examples
///
/// Must be constructed with the help of `Default` since new options might be added later.
///
/// ```
/// # use cosmwasm_std::Timestamp;
/// use cosmwasm_std::testing::{Envs, EnvsOptions};
///
/// let mut options = EnvsOptions::default();
/// options.chain_id = "megachain".to_string();
/// options.bech32_prefix = "mega";
/// let mut envs = Envs::with_options(options);
///
/// let env = envs.make();
/// assert_eq!(env.block.chain_id, "megachain");
/// assert_eq!(env.contract.address.as_str(), "mega1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922ts7vnj8h");
/// ```
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct EnvsOptions {
    pub bech32_prefix: &'static str, /* static due to MockApi's Copy requirement. No better idea for now. */
    pub block_time: u64,
    // The height before the first `make` call
    pub initial_height: u64,
    // The block time before the first `make` call
    pub initial_time: Timestamp,
    pub chain_id: String,
}

impl Default for EnvsOptions {
    fn default() -> Self {
        EnvsOptions {
            bech32_prefix: BECH32_PREFIX,
            block_time: 5_000_000_000, // 5s
            initial_height: 12_344,
            initial_time: Timestamp::from_nanos(1_571_797_419_879_305_533).minus_seconds(5),
            chain_id: "cosmos-testnet-14002".to_string(),
        }
    }
}

impl Envs {
    pub fn new(bech32_prefix: &'static str) -> Self {
        Self::with_options(EnvsOptions {
            bech32_prefix,
            ..Default::default()
        })
    }

    pub fn with_options(options: EnvsOptions) -> Self {
        let api = MockApi::default().with_prefix(options.bech32_prefix);
        Envs {
            chain_id: options.chain_id,
            // Default values here for compatibility with old `mock_env` function. They could be changed to anything else if there is a good reason.
            contract_address: api.addr_make("cosmos2contract"),
            block_time: options.block_time,
            last_height: options.initial_height,
            last_time: options.initial_time,
        }
    }

    pub fn make(&mut self) -> Env {
        self.checked_make().unwrap()
    }

    fn checked_make(&mut self) -> Option<Env> {
        let height = self.last_height.checked_add(1)?;
        let time = Timestamp::from_nanos(self.last_time.nanos().checked_add(self.block_time)?);

        self.last_height = height;
        self.last_time = time;

        Some(Env {
            block: BlockInfo {
                height,
                time,
                chain_id: self.chain_id.clone(),
            },
            transaction: Some(TransactionInfo::new(
                3,
                Binary::from_hex(
                    "E5469DACEC17CEF8A260FD37675ED87E7FB6A2B5AD95193C51308006C7E494B3",
                )
                .unwrap(),
            )),
            contract: ContractInfo {
                address: self.contract_address.clone(),
            },
        })
    }
}

impl Default for Envs {
    fn default() -> Self {
        Envs::with_options(EnvsOptions::default())
    }
}

// The iterator implementation ends in case of overflows to avoid panics.
// Using this is recommended for very long running test suites.
impl Iterator for Envs {
    type Item = Env;

    fn next(&mut self) -> Option<Self::Item> {
        self.checked_make()
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
            data: to_json_binary(data)?,
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
        Addr::unchecked("relayer"),
    ))
}

/// Creates a Ibc2PacketAckMsg for testing ibc2_packet_ack. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields
#[cfg(feature = "ibc2")]
pub fn mock_ibc2_packet_ack(data: &impl Serialize) -> StdResult<Ibc2PacketAckMsg> {
    Ok(Ibc2PacketAckMsg::new(
        "source_id23".to_string(),
        "channel_id23".to_string(),
        Ibc2Payload {
            source_port: "wasm2srcport".to_string(),
            destination_port: "wasm2destport".to_string(),
            version: "v2".to_string(),
            encoding: "json".to_string(),
            value: to_json_binary(data)?,
        },
        Binary::new(vec![]),
        Addr::unchecked("relayer"),
    ))
}

/// Creates a IbcPacketReceiveMsg for testing ibc_packet_receive. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields
#[cfg(feature = "ibc2")]
pub fn mock_ibc2_packet_recv(data: &impl Serialize) -> StdResult<Ibc2PacketReceiveMsg> {
    Ok(Ibc2PacketReceiveMsg::new(
        Ibc2Payload {
            source_port: "wasm2srcport".to_string(),
            destination_port: "wasm2destport".to_string(),
            version: "v2".to_string(),
            encoding: "json".to_string(),
            value: to_json_binary(data)?,
        },
        Addr::unchecked("relayer"),
        "channel_id23".to_string(),
        42,
    ))
}

/// Creates a Ibc2PacketTimeoutMsg for testing ibc2_packet_timeout.
#[cfg(feature = "ibc2")]
pub fn mock_ibc2_packet_timeout(data: &impl Serialize) -> StdResult<Ibc2PacketTimeoutMsg> {
    let payload = Ibc2Payload {
        source_port: "wasm2srcport".to_string(),
        destination_port: "wasm2destport".to_string(),
        version: "v2".to_string(),
        encoding: "json".to_string(),
        value: to_json_binary(data)?,
    };
    Ok(Ibc2PacketTimeoutMsg::new(
        payload,
        "source_client".to_string(),
        "destination_client".to_string(),
        1,
        Addr::unchecked("relayer"),
    ))
}

/// Creates a Ibc2PacketTimeoutMsg for testing ibc2_packet_timeout.
#[cfg(feature = "ibc2")]
pub fn mock_ibc2_packet_send(data: &impl Serialize) -> StdResult<Ibc2PacketSendMsg> {
    let payload = Ibc2Payload {
        source_port: "wasm2srcport".to_string(),
        destination_port: "wasm2destport".to_string(),
        version: "v2".to_string(),
        encoding: "json".to_string(),
        value: to_json_binary(data)?,
    };
    Ok(Ibc2PacketSendMsg::new(
        payload,
        "source_client".to_string(),
        "destination_client".to_string(),
        1,
        Addr::unchecked("signer_contract"),
    ))
}

/// Creates a IbcPacket for testing ibc_packet_{ack,timeout}. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields.
/// The difference from mock_ibc_packet_recv is if `my_channel_id` is src or dest.
#[cfg(feature = "stargate")]
fn mock_ibc_packet(my_channel_id: &str, data: &impl Serialize) -> StdResult<IbcPacket> {
    Ok(IbcPacket {
        data: to_json_binary(data)?,
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
        Addr::unchecked("relayer"),
    ))
}

/// Creates a IbcPacketTimeoutMsg for testing ibc_packet_timeout. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields.
/// The difference from mock_ibc_packet_recv is if `my_channel_id` is src or dest.
#[cfg(feature = "stargate")]
pub fn mock_ibc_packet_timeout(
    my_channel_id: &str,
    data: &impl Serialize,
) -> StdResult<IbcPacketTimeoutMsg> {
    let packet = mock_ibc_packet(my_channel_id, data)?;
    Ok(IbcPacketTimeoutMsg::new(packet, Addr::unchecked("relayer")))
}

/// The same type as cosmwasm-std's QuerierResult, but easier to reuse in
/// cosmwasm-vm. It might diverge from QuerierResult at some point.
pub type MockQuerierCustomHandlerResult = SystemResult<ContractResult<Binary>>;

/// MockQuerier holds an immutable table of bank balances
/// and configurable handlers for Wasm queries and custom queries.
pub struct MockQuerier<C: DeserializeOwned = Empty> {
    pub bank: BankQuerier,
    #[cfg(feature = "staking")]
    pub staking: StakingQuerier,
    #[cfg(feature = "cosmwasm_1_3")]
    pub distribution: DistributionQuerier,
    wasm: WasmQuerier,
    #[cfg(feature = "stargate")]
    pub ibc: IbcQuerier,
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
            #[cfg(feature = "cosmwasm_1_3")]
            distribution: DistributionQuerier::default(),
            #[cfg(feature = "staking")]
            staking: StakingQuerier::default(),
            wasm: WasmQuerier::default(),
            #[cfg(feature = "stargate")]
            ibc: IbcQuerier::default(),
            // strange argument notation suggested as a workaround here: https://github.com/rust-lang/rust/issues/41078#issuecomment-294296365
            custom_handler: Box::from(|_: &_| -> MockQuerierCustomHandlerResult {
                SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "custom".to_string(),
                })
            }),
        }
    }

    pub fn update_wasm<WH>(&mut self, handler: WH)
    where
        WH: Fn(&WasmQuery) -> QuerierResult + 'static,
    {
        self.wasm.update_handler(handler)
    }

    pub fn with_custom_handler<CH>(mut self, handler: CH) -> Self
    where
        CH: Fn(&C) -> MockQuerierCustomHandlerResult + 'static,
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
        let request: QueryRequest<C> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {e}"),
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
            #[cfg(feature = "cosmwasm_1_3")]
            QueryRequest::Distribution(distribution_query) => {
                self.distribution.query(distribution_query)
            }
            QueryRequest::Wasm(msg) => self.wasm.query(msg),
            #[cfg(feature = "stargate")]
            #[allow(deprecated)]
            QueryRequest::Stargate { .. } => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "Stargate".to_string(),
            }),
            #[cfg(feature = "cosmwasm_2_0")]
            QueryRequest::Grpc(_) => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "GRPC".to_string(),
            }),
            #[cfg(feature = "stargate")]
            QueryRequest::Ibc(msg) => self.ibc.query(msg),
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

    fn update_handler<WH>(&mut self, handler: WH)
    where
        WH: Fn(&WasmQuery) -> QuerierResult + 'static,
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
            let err = match request {
                WasmQuery::Smart { contract_addr, .. } => SystemError::NoSuchContract {
                    addr: contract_addr.clone(),
                },
                WasmQuery::Raw { contract_addr, .. } => SystemError::NoSuchContract {
                    addr: contract_addr.clone(),
                },
                WasmQuery::ContractInfo { contract_addr, .. } => SystemError::NoSuchContract {
                    addr: contract_addr.clone(),
                },
                #[cfg(feature = "cosmwasm_1_2")]
                WasmQuery::CodeInfo { code_id, .. } => {
                    SystemError::NoSuchCode { code_id: *code_id }
                }
                #[cfg(all(feature = "cosmwasm_3_0", feature = "iterator"))]
                WasmQuery::RawRange { contract_addr, .. } => SystemError::NoSuchContract {
                    addr: contract_addr.clone(),
                },
            };
            SystemResult::Err(err)
        });
        Self::new(handler)
    }
}

#[derive(Clone, Default)]
pub struct BankQuerier {
    #[allow(dead_code)]
    /// BTreeMap<denom, amount>
    supplies: BTreeMap<String, Uint256>,
    /// A map from address to balance. The address is the String conversion of `Addr`,
    /// i.e. the bech32 encoded address.
    balances: BTreeMap<String, Vec<Coin>>,
    /// Vec<Metadata>
    denom_metadata: BTreeMap<Vec<u8>, DenomMetadata>,
}

impl BankQuerier {
    pub fn new(balances: &[(&str, &[Coin])]) -> Self {
        let balances: BTreeMap<_, _> = balances
            .iter()
            .map(|(address, balance)| (address.to_string(), balance.to_vec()))
            .collect();

        BankQuerier {
            supplies: Self::calculate_supplies(&balances),
            balances,
            denom_metadata: BTreeMap::new(),
        }
    }

    /// set a new balance for the given address and return the old balance
    pub fn update_balance(
        &mut self,
        addr: impl Into<String>,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        let result = self.balances.insert(addr.into(), balance);
        self.supplies = Self::calculate_supplies(&self.balances);

        result
    }

    pub fn set_denom_metadata(&mut self, denom_metadata: &[DenomMetadata]) {
        self.denom_metadata = denom_metadata
            .iter()
            .map(|d| (d.base.as_bytes().to_vec(), d.clone()))
            .collect();
    }

    fn calculate_supplies(balances: &BTreeMap<String, Vec<Coin>>) -> BTreeMap<String, Uint256> {
        let mut supplies = BTreeMap::new();

        let all_coins = balances.iter().flat_map(|(_, coins)| coins);

        for coin in all_coins {
            *supplies
                .entry(coin.denom.clone())
                .or_insert_with(Uint256::zero) += coin.amount;
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
                    .unwrap_or_else(Uint256::zero);
                let bank_res = SupplyResponse {
                    amount: Coin {
                        amount,
                        denom: denom.to_string(),
                    },
                };
                to_json_binary(&bank_res).into()
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
                to_json_binary(&bank_res).into()
            }
            #[cfg(feature = "cosmwasm_1_3")]
            BankQuery::DenomMetadata { denom } => {
                let denom_metadata = self.denom_metadata.get(denom.as_bytes());
                match denom_metadata {
                    Some(m) => {
                        let metadata_res = DenomMetadataResponse {
                            metadata: m.clone(),
                        };
                        to_json_binary(&metadata_res).into()
                    }
                    None => return SystemResult::Err(SystemError::Unknown {}),
                }
            }
            #[cfg(feature = "cosmwasm_1_3")]
            BankQuery::AllDenomMetadata { pagination } => {
                let default_pagination = PageRequest {
                    key: None,
                    limit: 100,
                    reverse: false,
                };
                let pagination = pagination.as_ref().unwrap_or(&default_pagination);

                // range of all denoms after the given key (or until the key for reverse)
                let range = match (pagination.reverse, &pagination.key) {
                    (_, None) => (Bound::Unbounded, Bound::Unbounded),
                    (true, Some(key)) => (Bound::Unbounded, Bound::Included(key.as_slice())),
                    (false, Some(key)) => (Bound::Included(key.as_slice()), Bound::Unbounded),
                };
                let iter = self.denom_metadata.range::<[u8], _>(range);
                // using dynamic dispatch here to reduce code duplication and since this is only testing code
                let iter: Box<dyn Iterator<Item = _>> = if pagination.reverse {
                    Box::new(iter.rev())
                } else {
                    Box::new(iter)
                };

                let mut metadata: Vec<_> = iter
                    // take the requested amount + 1 to get the next key
                    .take((pagination.limit.saturating_add(1)) as usize)
                    .map(|(_, m)| m.clone())
                    .collect();

                // if we took more than requested, remove the last element (the next key),
                // otherwise this is the last batch
                let next_key = if metadata.len() > pagination.limit as usize {
                    metadata.pop().map(|m| Binary::from(m.base.as_bytes()))
                } else {
                    None
                };

                let metadata_res = AllDenomMetadataResponse { metadata, next_key };
                to_json_binary(&metadata_res).into()
            }
        };
        // system result is always ok in the mock implementation
        SystemResult::Ok(contract_result)
    }
}

#[cfg(feature = "stargate")]
#[derive(Clone, Default)]
pub struct IbcQuerier {
    port_id: String,
    channels: Vec<IbcChannel>,
}

#[cfg(feature = "stargate")]
impl IbcQuerier {
    /// Create a mock querier where:
    /// - port_id is the port the "contract" is bound to
    /// - channels are a list of ibc channels
    pub fn new(port_id: &str, channels: &[IbcChannel]) -> Self {
        IbcQuerier {
            port_id: port_id.to_string(),
            channels: channels.to_vec(),
        }
    }

    /// Update the querier's configuration
    pub fn update(&mut self, port_id: impl Into<String>, channels: &[IbcChannel]) {
        self.port_id = port_id.into();
        self.channels = channels.to_vec();
    }

    pub fn query(&self, request: &IbcQuery) -> QuerierResult {
        let contract_result: ContractResult<Binary> = match request {
            IbcQuery::Channel {
                channel_id,
                port_id,
            } => {
                let channel = self
                    .channels
                    .iter()
                    .find(|c| match port_id {
                        Some(p) => c.endpoint.channel_id.eq(channel_id) && c.endpoint.port_id.eq(p),
                        None => {
                            c.endpoint.channel_id.eq(channel_id)
                                && c.endpoint.port_id == self.port_id
                        }
                    })
                    .cloned();
                let res = ChannelResponse { channel };
                to_json_binary(&res).into()
            }
            IbcQuery::PortId {} => {
                let res = PortIdResponse {
                    port_id: self.port_id.clone(),
                };
                to_json_binary(&res).into()
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

    /// Update the querier's configuration
    pub fn update(
        &mut self,
        denom: impl Into<String>,
        validators: &[Validator],
        delegations: &[FullDelegation],
    ) {
        self.denom = denom.into();
        self.validators = validators.to_vec();
        self.delegations = delegations.to_vec();
    }

    pub fn query(&self, request: &StakingQuery) -> QuerierResult {
        let contract_result: ContractResult<Binary> = match request {
            StakingQuery::BondedDenom {} => {
                let res = BondedDenomResponse {
                    denom: self.denom.clone(),
                };
                to_json_binary(&res).into()
            }
            StakingQuery::AllValidators {} => {
                let res = AllValidatorsResponse {
                    validators: self.validators.iter().cloned().map(Into::into).collect(),
                };
                to_json_binary(&res).into()
            }
            StakingQuery::Validator { address } => {
                let validator: Option<Validator> = self
                    .validators
                    .iter()
                    .find(|validator| validator.address == *address)
                    .cloned();
                let res = ValidatorResponse { validator };
                to_json_binary(&res).into()
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
                to_json_binary(&res).into()
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
                to_json_binary(&res).into()
            }
        };
        // system result is always ok in the mock implementation
        SystemResult::Ok(contract_result)
    }
}

#[cfg(feature = "cosmwasm_1_3")]
#[derive(Clone, Default)]
pub struct DistributionQuerier {
    withdraw_addresses: BTreeMap<String, String>,
    /// Mock of accumulated rewards, indexed first by delegator and then validator address.
    rewards: BTreeMap<String, BTreeMap<String, Vec<DecCoin>>>,
    /// Mock of validators that a delegator has bonded to.
    validators: BTreeMap<String, BTreeSet<String>>,
}

#[cfg(feature = "cosmwasm_1_3")]
impl DistributionQuerier {
    pub fn new<T>(withdraw_addresses: T) -> Self
    where
        T: IntoIterator<Item = (String, String)>,
    {
        DistributionQuerier {
            withdraw_addresses: withdraw_addresses.into_iter().collect(),
            ..Default::default()
        }
    }

    pub fn set_withdraw_address(
        &mut self,
        delegator_address: impl Into<String>,
        withdraw_address: impl Into<String>,
    ) {
        self.withdraw_addresses
            .insert(delegator_address.into(), withdraw_address.into());
    }

    /// Sets multiple withdraw addresses.
    ///
    /// This allows passing multiple tuples of `(delegator_address, withdraw_address)`.
    /// It does not overwrite existing entries.
    pub fn set_withdraw_addresses(
        &mut self,
        withdraw_addresses: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) {
        for (d, w) in withdraw_addresses {
            self.set_withdraw_address(d, w);
        }
    }

    pub fn clear_withdraw_addresses(&mut self) {
        self.withdraw_addresses.clear();
    }

    /// Sets accumulated rewards for a given validator and delegator pair.
    pub fn set_rewards(
        &mut self,
        validator: impl Into<String>,
        delegator: impl Into<String>,
        rewards: Vec<DecCoin>,
    ) {
        self.rewards
            .entry(delegator.into())
            .or_default()
            .insert(validator.into(), rewards);
    }

    /// Sets the validators a given delegator has bonded to.
    pub fn set_validators(
        &mut self,
        delegator: impl Into<String>,
        validators: impl IntoIterator<Item = impl Into<String>>,
    ) {
        self.validators.insert(
            delegator.into(),
            validators.into_iter().map(Into::into).collect(),
        );
    }

    pub fn query(&self, request: &DistributionQuery) -> QuerierResult {
        let contract_result: ContractResult<Binary> = match request {
            DistributionQuery::DelegatorWithdrawAddress { delegator_address } => {
                let res = DelegatorWithdrawAddressResponse {
                    withdraw_address: Addr::unchecked(
                        self.withdraw_addresses
                            .get(delegator_address)
                            .unwrap_or(delegator_address),
                    ),
                };
                to_json_binary(&res).into()
            }
            #[cfg(feature = "cosmwasm_1_4")]
            DistributionQuery::DelegationRewards {
                delegator_address,
                validator_address,
            } => {
                let res = DelegationRewardsResponse {
                    rewards: self
                        .rewards
                        .get(delegator_address)
                        .and_then(|v| v.get(validator_address))
                        .cloned()
                        .unwrap_or_default(),
                };
                to_json_binary(&res).into()
            }
            #[cfg(feature = "cosmwasm_1_4")]
            DistributionQuery::DelegationTotalRewards { delegator_address } => {
                let validator_rewards = self
                    .validator_rewards(delegator_address)
                    .unwrap_or_default();
                let res = crate::DelegationTotalRewardsResponse {
                    total: validator_rewards
                        .iter()
                        .fold(BTreeMap::<&str, DecCoin>::new(), |mut acc, rewards| {
                            for coin in &rewards.reward {
                                acc.entry(&coin.denom)
                                    .or_insert_with(|| DecCoin {
                                        denom: coin.denom.clone(),
                                        amount: Decimal256::zero(),
                                    })
                                    .amount += coin.amount;
                            }

                            acc
                        })
                        .into_values()
                        .collect(),
                    rewards: validator_rewards,
                };
                to_json_binary(&res).into()
            }
            #[cfg(feature = "cosmwasm_1_4")]
            DistributionQuery::DelegatorValidators { delegator_address } => {
                let res = DelegatorValidatorsResponse {
                    validators: self
                        .validators
                        .get(delegator_address)
                        .map(|set| set.iter().cloned().collect())
                        .unwrap_or_default(),
                };
                to_json_binary(&res).into()
            }
        };
        // system result is always ok in the mock implementation
        SystemResult::Ok(contract_result)
    }

    /// Helper method to get all rewards for a given delegator.
    #[cfg(feature = "cosmwasm_1_4")]
    fn validator_rewards(&self, delegator_address: &str) -> Option<Vec<crate::DelegatorReward>> {
        let validator_rewards = self.rewards.get(delegator_address)?;

        Some(
            validator_rewards
                .iter()
                .map(|(validator, rewards)| crate::DelegatorReward {
                    validator_address: validator.clone(),
                    reward: rewards.clone(),
                })
                .collect(),
        )
    }
}

/// Only for test code. This bypasses assertions in new, allowing us to create _*
/// Attributes to simulate responses from the blockchain
pub fn mock_wasmd_attr(key: impl Into<String>, value: impl Into<String>) -> Attribute {
    Attribute {
        key: key.into(),
        value: value.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused)]
    use crate::coins;
    #[cfg(feature = "cosmwasm_1_3")]
    use crate::DenomUnit;
    use crate::{coin, instantiate2_address, ContractInfoResponse, HexBinary, Response, Storage};
    #[cfg(feature = "staking")]
    use crate::{Decimal, Delegation};
    use base64::{engine::general_purpose, Engine};
    use cosmwasm_core::BLS12_381_G1_GENERATOR;
    use hex_literal::hex;
    use serde::Deserialize;

    const SECP256K1_MSG_HASH_HEX: &str =
        "5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0";
    const SECP256K1_SIG_HEX: &str = "207082eb2c3dfa0b454e0906051270ba4074ac93760ba9e7110cd9471475111151eb0dbbc9920e72146fb564f99d039802bf6ef2561446eb126ef364d21ee9c4";
    const SECP256K1_PUBKEY_HEX: &str = "04051c1ee2190ecfb174bfe4f90763f2b4ff7517b70a2aec1876ebcfd644c4633fb03f3cfbd94b1f376e34592d9d41ccaf640bb751b00a1fadeb0c01157769eb73";

    const SECP256R1_MSG_HASH_HEX: &str =
        "5eb28029ebf3c7025ff2fc2f6de6f62aecf6a72139e1cba5f20d11bbef036a7f";
    const SECP256R1_SIG_HEX: &str = "e67a9717ccf96841489d6541f4f6adb12d17b59a6bef847b6183b8fcf16a32eb9ae6ba6d637706849a6a9fc388cf0232d85c26ea0d1fe7437adb48de58364333";
    const SECP256R1_PUBKEY_HEX: &str = "0468229b48c2fe19d3db034e4c15077eb7471a66031f28a980821873915298ba76303e8ee3742a893f78b810991da697083dd8f11128c47651c27a56740a80c24c";

    const ED25519_MSG_HEX: &str = "72";
    const ED25519_SIG_HEX: &str = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00";
    const ED25519_PUBKEY_HEX: &str =
        "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";

    // See https://github.com/drand/kyber-bls12381/issues/22 and
    // https://github.com/drand/drand/pull/1249
    const DOMAIN_HASH_TO_G2: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

    /// Public key League of Entropy Mainnet (curl -sS https://drand.cloudflare.com/info)
    const PK_LEO_MAINNET: [u8; 48] = hex!("868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31");

    const ETH_BLOCK_HEADER: &[u8] =
        include_bytes!("../../../crypto/testdata/eth-headers/1699693797.394876721s.json");

    #[test]
    fn mock_env_works() {
        let env = mock_env();
        assert_eq!(
            env,
            Env {
                block: BlockInfo {
                    height: 12345,
                    time: Timestamp::from_nanos(1571797419879305533),
                    chain_id: "cosmos-testnet-14002".to_string()
                },
                transaction: Some(TransactionInfo::new(
                    3,
                    Binary::from_hex(
                        "E5469DACEC17CEF8A260FD37675ED87E7FB6A2B5AD95193C51308006C7E494B3"
                    )
                    .unwrap(),
                )),
                contract: ContractInfo {
                    address: Addr::unchecked(MOCK_CONTRACT_ADDR)
                }
            }
        )
    }

    #[test]
    fn envs_works() {
        let mut envs = Envs::new("food");

        let env = envs.make();
        assert_eq!(
            env.contract.address.as_str(),
            "food1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922ts74yrjj"
        );
        assert_eq!(env.block.height, 12_345);
        assert_eq!(
            env.block.time,
            Timestamp::from_nanos(1_571_797_419_879_305_533)
        );

        let env = envs.make();
        assert_eq!(
            env.contract.address.as_str(),
            "food1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922ts74yrjj"
        );
        assert_eq!(env.block.height, 12_346);
        assert_eq!(
            env.block.time,
            Timestamp::from_nanos(1_571_797_424_879_305_533)
        );

        let env = envs.make();
        assert_eq!(
            env.contract.address.as_str(),
            "food1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922ts74yrjj"
        );
        assert_eq!(env.block.height, 12_347);
        assert_eq!(
            env.block.time,
            Timestamp::from_nanos(1_571_797_429_879_305_533)
        );
    }

    #[test]
    fn envs_implements_iterator() {
        let envs = Envs::new("food");

        let result: Vec<_> = envs.into_iter().take(5).collect();

        assert_eq!(
            result[0].contract.address.as_str(),
            "food1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922ts74yrjj"
        );
        assert_eq!(result[0].block.height, 12_345);
        assert_eq!(
            result[0].block.time,
            Timestamp::from_nanos(1_571_797_419_879_305_533)
        );

        assert_eq!(
            result[4].contract.address.as_str(),
            "food1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922ts74yrjj"
        );
        assert_eq!(result[4].block.height, 12_349);
        assert_eq!(
            result[4].block.time,
            Timestamp::from_nanos(1_571_797_439_879_305_533)
        );

        // Get a millions envs through iterator
        let mut envs = Envs::new("yo");
        let first = envs.next().unwrap();
        let last = envs.take(1_000_000).last().unwrap();
        assert_eq!(first.block.height, 12_345);
        assert_eq!(last.block.height, 1_012_345);
        assert_eq!(
            last.block.time,
            first.block.time.plus_seconds(1_000_000 * 5)
        );
    }

    #[test]
    fn addr_validate_works() {
        // default prefix is 'cosmwasm'
        let api = MockApi::default();

        // valid
        let humanized = "cosmwasm1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqs8s7vcp";
        let addr = api.addr_validate(humanized).unwrap();
        assert_eq!(addr.as_str(), humanized);

        // invalid: too short
        api.addr_validate("").unwrap_err();
        // invalid: not normalized
        api.addr_validate("Foobar123").unwrap_err();
        api.addr_validate("FOOBAR123").unwrap_err();
    }

    #[test]
    fn addr_canonicalize_works() {
        let api = MockApi::default();

        api.addr_canonicalize(
            "cosmwasm1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqs8s7vcp",
        )
        .unwrap();

        // is case insensitive
        let data1 = api
            .addr_canonicalize(
                "cosmwasm1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqs8s7vcp",
            )
            .unwrap();
        let data2 = api
            .addr_canonicalize(
                "COSMWASM1H34LMPYWH4UPNJDG90CJF4J70AEE6Z8QQFSPUGAMJP42E4Q28KQS8S7VCP",
            )
            .unwrap();
        assert_eq!(data1, data2);
    }

    #[test]
    fn canonicalize_and_humanize_restores_original() {
        // create api with 'cosmwasm' prefix
        let api = MockApi::default();

        // normalizes input
        let original =
            String::from("COSMWASM1H34LMPYWH4UPNJDG90CJF4J70AEE6Z8QQFSPUGAMJP42E4Q28KQS8S7VCP");
        let canonical = api.addr_canonicalize(&original).unwrap();
        let recovered = api.addr_humanize(&canonical).unwrap();
        assert_eq!(
            recovered.as_str(),
            "cosmwasm1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqs8s7vcp"
        );

        // create api with 'juno' prefix
        let api = MockApi::default().with_prefix("juno");

        // long input (Juno contract address)
        let original =
            String::from("juno1v82su97skv6ucfqvuvswe0t5fph7pfsrtraxf0x33d8ylj5qnrysdvkc95");
        let canonical = api.addr_canonicalize(&original).unwrap();
        let recovered = api.addr_humanize(&canonical).unwrap();
        assert_eq!(recovered.as_str(), original);
    }

    #[test]
    fn addr_canonicalize_short_input() {
        let api = MockApi::default();

        // empty address should fail
        let empty = "cosmwasm1pj90vm";
        assert!(api
            .addr_canonicalize(empty)
            .unwrap_err()
            .to_string()
            .contains("Invalid canonical address length"));

        // one byte address should work
        let human = "cosmwasm1qqvk2mde";
        assert_eq!(api.addr_canonicalize(human).unwrap().as_ref(), [0u8]);
    }

    #[test]
    fn addr_canonicalize_long_input() {
        let api = MockApi::default();
        let human =
            "cosmwasm1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqehqqkz";
        let err = api.addr_canonicalize(human).unwrap_err();
        assert!(err.to_string().contains("Invalid canonical address length"));
    }

    #[test]
    fn addr_humanize_input_length() {
        let api = MockApi::default();
        let input = CanonicalAddr::from(vec![]);
        assert_eq!(
            api.addr_humanize(&input).unwrap_err(),
            StdError::generic_err("Invalid canonical address length")
        );
    }

    #[test]
    fn bls12_381_aggregate_g1_works() {
        #[derive(serde::Deserialize)]
        struct EthHeader {
            public_keys: Vec<String>,
            aggregate_pubkey: String,
        }

        let api = MockApi::default();
        let header: EthHeader = serde_json::from_slice(ETH_BLOCK_HEADER).unwrap();
        let expected = general_purpose::STANDARD
            .decode(header.aggregate_pubkey)
            .unwrap();

        let pubkeys: Vec<u8> = header
            .public_keys
            .into_iter()
            .flat_map(|key| general_purpose::STANDARD.decode(key).unwrap())
            .collect();
        let sum = api.bls12_381_aggregate_g1(&pubkeys).unwrap();

        assert_eq!(expected, sum);
    }

    #[test]
    fn bls12_381_aggregate_g2_works() {
        let api = MockApi::default();

        let points: Vec<u8> = [
            hex!("b6ed936746e01f8ecf281f020953fbf1f01debd5657c4a383940b020b26507f6076334f91e2366c96e9ab279fb5158090352ea1c5b0c9274504f4f0e7053af24802e51e4568d164fe986834f41e55c8e850ce1f98458c0cfc9ab380b55285a55"),
            hex!("b23c46be3a001c63ca711f87a005c200cc550b9429d5f4eb38d74322144f1b63926da3388979e5321012fb1a0526bcd100b5ef5fe72628ce4cd5e904aeaa3279527843fae5ca9ca675f4f51ed8f83bbf7155da9ecc9663100a885d5dc6df96d9"),
            hex!("948a7cb99f76d616c2c564ce9bf4a519f1bea6b0a624a02276443c245854219fabb8d4ce061d255af5330b078d5380681751aa7053da2c98bae898edc218c75f07e24d8802a17cd1f6833b71e58f5eb5b94208b4d0bb3848cecb075ea21be115"),
        ]
        .into_iter()
        .flatten()
        .collect();

        let expected = hex!("9683b3e6701f9a4b706709577963110043af78a5b41991b998475a3d3fd62abf35ce03b33908418efc95a058494a8ae504354b9f626231f6b3f3c849dfdeaf5017c4780e2aee1850ceaf4b4d9ce70971a3d2cfcd97b7e5ecf6759f8da5f76d31");
        let sum = api.bls12_381_aggregate_g2(&points).unwrap();

        assert_eq!(sum, expected);
    }

    #[test]
    fn bls12_381_pairing_equality_works() {
        let api = MockApi::default();

        let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
        let ps = hex!("a491d1b0ecd9bb917989f0e74f0dea0422eac4a873e5e2644f368dffb9a6e20fd6e10c1b77654d067c0618f6e5a7f79ab301803f8b5ac4a1133581fc676dfedc60d891dd5fa99028805e5ea5b08d3491af75d0707adab3b70c6a6a580217bf81b53d21a4cfd562c469cc81514d4ce5a6b577d8403d32a394dc265dd190b47fa9f829fdd7963afdf972e5e77854051f6f");
        let qs: Vec<u8> = [
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("5656565656565656565656565656565656565656565656565656565656565656"),
            hex!("abababababababababababababababababababababababababababababababab"),
        ]
        .into_iter()
        .flat_map(|msg| {
            api.bls12_381_hash_to_g2(HashFunction::Sha256, &msg, dst)
                .unwrap()
        })
        .collect();
        let s = hex!("9104e74b9dfd3ad502f25d6a5ef57db0ed7d9a0e00f3500586d8ce44231212542fcfaf87840539b398bf07626705cf1105d246ca1062c6c2e1a53029a0f790ed5e3cb1f52f8234dc5144c45fc847c0cd37a92d68e7c5ba7c648a8a339f171244");

        let is_valid = api
            .bls12_381_pairing_equality(&ps, &qs, &BLS12_381_G1_GENERATOR, &s)
            .unwrap();
        assert!(is_valid);
    }

    #[test]
    fn bls12_381_hash_to_g1_works() {
        // See: <https://datatracker.ietf.org/doc/rfc9380/>; Section J.9.1

        let api = MockApi::default();
        let msg = b"abc";
        let dst = b"QUUX-V01-CS02-with-BLS12381G1_XMD:SHA-256_SSWU_RO_";

        let hashed_point = api
            .bls12_381_hash_to_g1(HashFunction::Sha256, msg, dst)
            .unwrap();
        let mut serialized_expected_compressed = hex!("03567bc5ef9c690c2ab2ecdf6a96ef1c139cc0b2f284dca0a9a7943388a49a3aee664ba5379a7655d3c68900be2f6903");
        // Set the compression tag
        serialized_expected_compressed[0] |= 0b1000_0000;

        assert_eq!(hashed_point, serialized_expected_compressed);
    }

    #[test]
    fn bls12_381_hash_to_g2_works() {
        let api = MockApi::default();
        let msg = b"abc";
        let dst = b"QUUX-V01-CS02-with-BLS12381G2_XMD:SHA-256_SSWU_RO_";

        let hashed_point = api
            .bls12_381_hash_to_g2(HashFunction::Sha256, msg, dst)
            .unwrap();
        let mut serialized_expected_compressed = hex!("139cddbccdc5e91b9623efd38c49f81a6f83f175e80b06fc374de9eb4b41dfe4ca3a230ed250fbe3a2acf73a41177fd802c2d18e033b960562aae3cab37a27ce00d80ccd5ba4b7fe0e7a210245129dbec7780ccc7954725f4168aff2787776e6");
        // Set the compression tag
        serialized_expected_compressed[0] |= 0b1000_0000;

        assert_eq!(hashed_point, serialized_expected_compressed);
    }

    #[test]
    fn bls12_318_pairing_equality_works() {
        fn build_bls_message(round: u64, previous_signature: &[u8]) -> Vec<u8> {
            Sha256::new()
                .chain_update(previous_signature)
                .chain_update(round.to_be_bytes())
                .finalize()
                .to_vec()
        }

        let api = MockApi::default();

        let previous_signature = hex::decode("a609e19a03c2fcc559e8dae14900aaefe517cb55c840f6e69bc8e4f66c8d18e8a609685d9917efbfb0c37f058c2de88f13d297c7e19e0ab24813079efe57a182554ff054c7638153f9b26a60e7111f71a0ff63d9571704905d3ca6df0b031747").unwrap();
        let signature = hex::decode("82f5d3d2de4db19d40a6980e8aa37842a0e55d1df06bd68bddc8d60002e8e959eb9cfa368b3c1b77d18f02a54fe047b80f0989315f83b12a74fd8679c4f12aae86eaf6ab5690b34f1fddd50ee3cc6f6cdf59e95526d5a5d82aaa84fa6f181e42").unwrap();
        let round: u64 = 72785;

        let msg = build_bls_message(round, &previous_signature);
        let msg_point = api
            .bls12_381_hash_to_g2(HashFunction::Sha256, &msg, DOMAIN_HASH_TO_G2)
            .unwrap();

        let is_valid = api
            .bls12_381_pairing_equality(
                &BLS12_381_G1_GENERATOR,
                &signature,
                &PK_LEO_MAINNET,
                &msg_point,
            )
            .unwrap();

        assert!(is_valid);
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
            err => panic!("Unexpected error: {err:?}"),
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
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    // Basic "works" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn secp256r1_verify_works() {
        let api = MockApi::default();

        let hash = hex::decode(SECP256R1_MSG_HASH_HEX).unwrap();
        let signature = hex::decode(SECP256R1_SIG_HEX).unwrap();
        let public_key = hex::decode(SECP256R1_PUBKEY_HEX).unwrap();

        assert!(api
            .secp256r1_verify(&hash, &signature, &public_key)
            .unwrap());
    }

    // Basic "fails" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn secp256r1_verify_fails() {
        let api = MockApi::default();

        let mut hash = hex::decode(SECP256R1_MSG_HASH_HEX).unwrap();
        // alter hash
        hash[0] ^= 0x01;
        let signature = hex::decode(SECP256R1_SIG_HEX).unwrap();
        let public_key = hex::decode(SECP256R1_PUBKEY_HEX).unwrap();

        assert!(!api
            .secp256r1_verify(&hash, &signature, &public_key)
            .unwrap());
    }

    // Basic "errors" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn secp256r1_verify_errs() {
        let api = MockApi::default();

        let hash = hex::decode(SECP256R1_MSG_HASH_HEX).unwrap();
        let signature = hex::decode(SECP256R1_SIG_HEX).unwrap();
        let public_key = vec![];

        let res = api.secp256r1_verify(&hash, &signature, &public_key);
        assert_eq!(res.unwrap_err(), VerificationError::InvalidPubkeyFormat);
    }

    #[test]
    fn secp256r1_recover_pubkey_works() {
        let api = MockApi::default();

        let hash = hex!("17b03f9f00f6692ccdde485fc63c4530751ef35da6f71336610944b0894fcfb8");
        let signature = hex!("9886ae46c1415c3bc959e82b760ad760aab66885a84e620aa339fdf102465c422bf3a80bc04faa35ebecc0f4864ac02d349f6f126e0f988501b8d3075409a26c");
        let recovery_param = 0;
        let expected = hex!("0451f99d2d52d4a6e734484a018b7ca2f895c2929b6754a3a03224d07ae61166ce4737da963c6ef7247fb88d19f9b0c667cac7fe12837fdab88c66f10d3c14cad1");

        let pubkey = api
            .secp256r1_recover_pubkey(&hash, &signature, recovery_param)
            .unwrap();
        assert_eq!(pubkey, expected);
    }

    #[test]
    fn secp256r1_recover_pubkey_fails_for_wrong_recovery_param() {
        let api = MockApi::default();

        let hash = hex!("17b03f9f00f6692ccdde485fc63c4530751ef35da6f71336610944b0894fcfb8");
        let signature = hex!("9886ae46c1415c3bc959e82b760ad760aab66885a84e620aa339fdf102465c422bf3a80bc04faa35ebecc0f4864ac02d349f6f126e0f988501b8d3075409a26c");
        let expected = hex!("0451f99d2d52d4a6e734484a018b7ca2f895c2929b6754a3a03224d07ae61166ce4737da963c6ef7247fb88d19f9b0c667cac7fe12837fdab88c66f10d3c14cad1");

        // Wrong recovery param leads to different pubkey
        let pubkey = api.secp256r1_recover_pubkey(&hash, &signature, 1).unwrap();
        assert_eq!(pubkey.len(), 65);
        assert_ne!(pubkey, expected);

        // Invalid recovery param leads to error
        let result = api.secp256r1_recover_pubkey(&hash, &signature, 42);
        match result.unwrap_err() {
            RecoverPubkeyError::InvalidRecoveryParam => {}
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn secp256r1_recover_pubkey_fails_for_wrong_hash() {
        let api = MockApi::default();

        let hash = hex!("17b03f9f00f6692ccdde485fc63c4530751ef35da6f71336610944b0894fcfb8");
        let signature = hex!("9886ae46c1415c3bc959e82b760ad760aab66885a84e620aa339fdf102465c422bf3a80bc04faa35ebecc0f4864ac02d349f6f126e0f988501b8d3075409a26c");
        let recovery_param = 0;
        let expected = hex!("0451f99d2d52d4a6e734484a018b7ca2f895c2929b6754a3a03224d07ae61166ce4737da963c6ef7247fb88d19f9b0c667cac7fe12837fdab88c66f10d3c14cad1");

        // Wrong hash
        let mut corrupted_hash = hash;
        corrupted_hash[0] ^= 0x01;
        let pubkey = api
            .secp256r1_recover_pubkey(&corrupted_hash, &signature, recovery_param)
            .unwrap();
        assert_eq!(pubkey.len(), 65);
        assert_ne!(pubkey, expected);

        // Malformed hash
        let mut malformed_hash = hash.to_vec();
        malformed_hash.push(0x8a);
        let result = api.secp256r1_recover_pubkey(&malformed_hash, &signature, recovery_param);
        match result.unwrap_err() {
            RecoverPubkeyError::InvalidHashFormat => {}
            err => panic!("Unexpected error: {err:?}"),
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
        let res: SupplyResponse = from_json(elf).unwrap();
        assert_eq!(res.amount, coin(444, "ELF"));

        let fly = bank
            .query(&BankQuery::Supply {
                denom: "FLY".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: SupplyResponse = from_json(fly).unwrap();
        assert_eq!(res.amount, coin(777, "FLY"));

        // if a denom does not exist, should return zero amount, instead of throwing an error
        let atom = bank
            .query(&BankQuery::Supply {
                denom: "ATOM".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: SupplyResponse = from_json(atom).unwrap();
        assert_eq!(res.amount, coin(0, "ATOM"));
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
        let res: BalanceResponse = from_json(fly).unwrap();
        assert_eq!(res.amount, coin(777, "FLY"));

        // missing denom
        let miss = bank
            .query(&BankQuery::Balance {
                address: addr,
                denom: "MISS".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_json(miss).unwrap();
        assert_eq!(res.amount, coin(0, "MISS"));
    }

    #[test]
    #[allow(deprecated)]
    fn bank_querier_missing_account() {
        let addr = String::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let bank = BankQuerier::new(&[(&addr, &balance)]);

        // any denom on balances on empty account is empty coin
        let miss = bank
            .query(&BankQuery::Balance {
                address: String::from("elsewhere"),
                denom: "ELF".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_json(miss).unwrap();
        assert_eq!(res.amount, coin(0, "ELF"));
    }

    #[cfg(feature = "cosmwasm_1_3")]
    #[test]
    fn bank_querier_metadata_works() {
        let mut bank = BankQuerier::new(&[]);
        bank.set_denom_metadata(
            &(0..100)
                .map(|i| DenomMetadata {
                    symbol: format!("FOO{i}"),
                    name: "Foo".to_string(),
                    description: "Foo coin".to_string(),
                    denom_units: vec![DenomUnit {
                        denom: "ufoo".to_string(),
                        exponent: 8,
                        aliases: vec!["microfoo".to_string(), "foobar".to_string()],
                    }],
                    display: "FOO".to_string(),
                    base: format!("ufoo{i}"),
                    uri: "https://foo.bar".to_string(),
                    uri_hash: "foo".to_string(),
                })
                .collect::<Vec<_>>(),
        );

        // querying first 10 should work
        let res = bank
            .query(&BankQuery::AllDenomMetadata {
                pagination: Some(PageRequest {
                    key: None,
                    limit: 10,
                    reverse: false,
                }),
            })
            .unwrap()
            .unwrap();
        let res: AllDenomMetadataResponse = from_json(res).unwrap();
        assert_eq!(res.metadata.len(), 10);
        assert!(res.next_key.is_some());

        // querying next 10 should also work
        let res2 = bank
            .query(&BankQuery::AllDenomMetadata {
                pagination: Some(PageRequest {
                    key: res.next_key,
                    limit: 10,
                    reverse: false,
                }),
            })
            .unwrap()
            .unwrap();
        let res2: AllDenomMetadataResponse = from_json(res2).unwrap();
        assert_eq!(res2.metadata.len(), 10);
        assert_ne!(res.metadata.last(), res2.metadata.first());
        // should have no overlap
        for m in res.metadata {
            assert!(!res2.metadata.contains(&m));
        }

        // querying all 100 should work
        let res = bank
            .query(&BankQuery::AllDenomMetadata {
                pagination: Some(PageRequest {
                    key: None,
                    limit: 100,
                    reverse: true,
                }),
            })
            .unwrap()
            .unwrap();
        let res: AllDenomMetadataResponse = from_json(res).unwrap();
        assert_eq!(res.metadata.len(), 100);
        assert!(res.next_key.is_none(), "no more data should be available");
        assert_eq!(res.metadata[0].symbol, "FOO99", "should have been reversed");

        let more_res = bank
            .query(&BankQuery::AllDenomMetadata {
                pagination: Some(PageRequest {
                    key: res.next_key,
                    limit: u32::MAX,
                    reverse: true,
                }),
            })
            .unwrap()
            .unwrap();
        let more_res: AllDenomMetadataResponse = from_json(more_res).unwrap();
        assert_eq!(
            more_res.metadata, res.metadata,
            "should be same as previous query"
        );
    }

    #[cfg(feature = "cosmwasm_1_3")]
    #[test]
    fn distribution_querier_delegator_withdraw_address() {
        let mut distribution = DistributionQuerier::default();
        distribution.set_withdraw_address("addr0", "withdraw0");

        let query = DistributionQuery::DelegatorWithdrawAddress {
            delegator_address: "addr0".to_string(),
        };

        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegatorWithdrawAddressResponse = from_json(res).unwrap();
        assert_eq!(res.withdraw_address.as_str(), "withdraw0");

        let query = DistributionQuery::DelegatorWithdrawAddress {
            delegator_address: "addr1".to_string(),
        };

        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegatorWithdrawAddressResponse = from_json(res).unwrap();
        assert_eq!(res.withdraw_address.as_str(), "addr1");
    }

    #[cfg(feature = "cosmwasm_1_4")]
    #[test]
    fn distribution_querier_delegator_validators() {
        let mut distribution = DistributionQuerier::default();
        distribution.set_validators("addr0", ["valoper1", "valoper2"]);

        let query = DistributionQuery::DelegatorValidators {
            delegator_address: "addr0".to_string(),
        };

        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegatorValidatorsResponse = from_json(res).unwrap();
        assert_eq!(res.validators, ["valoper1", "valoper2"]);

        let query = DistributionQuery::DelegatorValidators {
            delegator_address: "addr1".to_string(),
        };

        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegatorValidatorsResponse = from_json(res).unwrap();
        assert_eq!(res.validators, ([] as [String; 0]));
    }

    #[cfg(feature = "cosmwasm_1_4")]
    #[test]
    fn distribution_querier_delegation_rewards() {
        use crate::{Decimal256, DelegationTotalRewardsResponse, DelegatorReward};

        let mut distribution = DistributionQuerier::default();
        let valoper0_rewards = vec![
            DecCoin::new(Decimal256::from_atomics(1234u128, 0).unwrap(), "uatom"),
            DecCoin::new(Decimal256::from_atomics(56781234u128, 4).unwrap(), "utest"),
        ];
        distribution.set_rewards("valoper0", "addr0", valoper0_rewards.clone());

        // both exist / are set
        let query = DistributionQuery::DelegationRewards {
            delegator_address: "addr0".to_string(),
            validator_address: "valoper0".to_string(),
        };
        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegationRewardsResponse = from_json(res).unwrap();
        assert_eq!(res.rewards, valoper0_rewards);

        // delegator does not exist
        let query = DistributionQuery::DelegationRewards {
            delegator_address: "nonexistent".to_string(),
            validator_address: "valoper0".to_string(),
        };
        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegationRewardsResponse = from_json(res).unwrap();
        assert_eq!(res.rewards.len(), 0);

        // validator does not exist
        let query = DistributionQuery::DelegationRewards {
            delegator_address: "addr0".to_string(),
            validator_address: "valopernonexistent".to_string(),
        };
        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegationRewardsResponse = from_json(res).unwrap();
        assert_eq!(res.rewards.len(), 0);

        // add one more validator
        let valoper1_rewards = vec![DecCoin::new(Decimal256::one(), "uatom")];
        distribution.set_rewards("valoper1", "addr0", valoper1_rewards.clone());

        // total rewards
        let query = DistributionQuery::DelegationTotalRewards {
            delegator_address: "addr0".to_string(),
        };
        let res = distribution.query(&query).unwrap().unwrap();
        let res: DelegationTotalRewardsResponse = from_json(res).unwrap();
        assert_eq!(
            res.rewards,
            vec![
                DelegatorReward {
                    validator_address: "valoper0".into(),
                    reward: valoper0_rewards
                },
                DelegatorReward {
                    validator_address: "valoper1".into(),
                    reward: valoper1_rewards
                },
            ]
        );
        assert_eq!(
            res.total,
            [
                DecCoin::new(
                    Decimal256::from_atomics(1234u128, 0).unwrap() + Decimal256::one(),
                    "uatom"
                ),
                // total for utest should still be the same
                DecCoin::new(Decimal256::from_atomics(56781234u128, 4).unwrap(), "utest")
            ]
        );
    }

    #[cfg(feature = "stargate")]
    #[test]
    fn ibc_querier_channel_existing() {
        let chan1 = mock_ibc_channel("channel-0", IbcOrder::Ordered, "ibc");
        let chan2 = mock_ibc_channel("channel-1", IbcOrder::Ordered, "ibc");

        let ibc = IbcQuerier::new("myport", &[chan1.clone(), chan2]);

        // query existing
        let query = &IbcQuery::Channel {
            channel_id: "channel-0".to_string(),
            port_id: Some("my_port".to_string()),
        };
        let raw = ibc.query(query).unwrap().unwrap();
        let chan: ChannelResponse = from_json(raw).unwrap();
        assert_eq!(chan.channel, Some(chan1));
    }

    #[cfg(feature = "stargate")]
    #[test]
    fn ibc_querier_channel_existing_no_port() {
        let chan1 = IbcChannel {
            endpoint: IbcEndpoint {
                port_id: "myport".to_string(),
                channel_id: "channel-0".to_string(),
            },
            counterparty_endpoint: IbcEndpoint {
                port_id: "their_port".to_string(),
                channel_id: "channel-7".to_string(),
            },
            order: IbcOrder::Ordered,
            version: "ibc".to_string(),
            connection_id: "connection-2".to_string(),
        };
        let chan2 = mock_ibc_channel("channel-1", IbcOrder::Ordered, "ibc");

        let ibc = IbcQuerier::new("myport", &[chan1.clone(), chan2]);

        // query existing
        let query = &IbcQuery::Channel {
            channel_id: "channel-0".to_string(),
            port_id: Some("myport".to_string()),
        };
        let raw = ibc.query(query).unwrap().unwrap();
        let chan: ChannelResponse = from_json(raw).unwrap();
        assert_eq!(chan.channel, Some(chan1));
    }

    #[cfg(feature = "stargate")]
    #[test]
    fn ibc_querier_channel_none() {
        let chan1 = mock_ibc_channel("channel-0", IbcOrder::Ordered, "ibc");
        let chan2 = mock_ibc_channel("channel-1", IbcOrder::Ordered, "ibc");

        let ibc = IbcQuerier::new("myport", &[chan1, chan2]);

        // query non-existing
        let query = &IbcQuery::Channel {
            channel_id: "channel-0".to_string(),
            port_id: None,
        };
        let raw = ibc.query(query).unwrap().unwrap();
        let chan: ChannelResponse = from_json(raw).unwrap();
        assert_eq!(chan.channel, None);
    }

    #[cfg(feature = "stargate")]
    #[test]
    fn ibc_querier_port() {
        let chan1 = mock_ibc_channel("channel-0", IbcOrder::Ordered, "ibc");

        let ibc = IbcQuerier::new("myport", &[chan1]);

        // query channels matching "myport" (should be none)
        let query = &IbcQuery::PortId {};
        let raw = ibc.query(query).unwrap().unwrap();
        let res: PortIdResponse = from_json(raw).unwrap();
        assert_eq!(res.port_id, "myport");
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
        let vals: AllValidatorsResponse = from_json(raw).unwrap();
        assert_eq!(vals.validators, vec![val1.into(), val2.into()]);
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
        let res: ValidatorResponse = from_json(raw).unwrap();
        assert_eq!(res.validator, Some(val1));

        // query 2
        let raw = staking
            .query(&StakingQuery::Validator { address: address2 })
            .unwrap()
            .unwrap();
        let res: ValidatorResponse = from_json(raw).unwrap();
        assert_eq!(res.validator, Some(val2));

        // query non-existent
        let raw = staking
            .query(&StakingQuery::Validator {
                address: address_non_existent,
            })
            .unwrap()
            .unwrap();
        let res: ValidatorResponse = from_json(raw).unwrap();
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
        let dels: AllDelegationsResponse = from_json(raw).unwrap();
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
        let dels: DelegationResponse = from_json(raw).unwrap();
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

        // By default, querier errors for WasmQuery::Raw
        let system_err = querier
            .query(&WasmQuery::Raw {
                contract_addr: any_addr.clone(),
                key: b"the key".into(),
            })
            .unwrap_err();
        match system_err {
            SystemError::NoSuchContract { addr } => assert_eq!(addr, any_addr),
            err => panic!("Unexpected error: {err:?}"),
        }

        // By default, querier errors for WasmQuery::Smart
        let system_err = querier
            .query(&WasmQuery::Smart {
                contract_addr: any_addr.clone(),
                msg: b"{}".into(),
            })
            .unwrap_err();
        match system_err {
            SystemError::NoSuchContract { addr } => assert_eq!(addr, any_addr),
            err => panic!("Unexpected error: {err:?}"),
        }

        // By default, querier errors for WasmQuery::ContractInfo
        let system_err = querier
            .query(&WasmQuery::ContractInfo {
                contract_addr: any_addr.clone(),
            })
            .unwrap_err();
        match system_err {
            SystemError::NoSuchContract { addr } => assert_eq!(addr, any_addr),
            err => panic!("Unexpected error: {err:?}"),
        }

        #[cfg(feature = "cosmwasm_1_2")]
        {
            // By default, querier errors for WasmQuery::CodeInfo
            let system_err = querier
                .query(&WasmQuery::CodeInfo { code_id: 4 })
                .unwrap_err();
            match system_err {
                SystemError::NoSuchCode { code_id } => assert_eq!(code_id, 4),
                err => panic!("Unexpected error: {err:?}"),
            }
        }

        #[cfg(all(feature = "cosmwasm_3_0", feature = "iterator"))]
        {
            // By default, querier errors for WasmQuery::RawRange
            let system_err = querier
                .query(&WasmQuery::RawRange {
                    contract_addr: any_addr.clone(),
                    start: None,
                    end: None,
                    limit: 10,
                    order: crate::Order::Ascending,
                })
                .unwrap_err();
            match system_err {
                SystemError::NoSuchContract { addr } => assert_eq!(addr, any_addr),
                err => panic!("Unexpected error: {err:?}"),
            }
        }

        querier.update_handler(|request| {
            let api = MockApi::default();
            let contract1 = api.addr_make("contract1");
            let mut storage1 = MockStorage::new();
            storage1.set(b"the key", b"the value");

            match request {
                WasmQuery::Raw { contract_addr, key } => {
                    let Ok(addr) = api.addr_validate(contract_addr) else {
                        return SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        });
                    };
                    if addr == contract1 {
                        if let Some(value) = storage1.get(key) {
                            SystemResult::Ok(ContractResult::Ok(Binary::new(value)))
                        } else {
                            SystemResult::Ok(ContractResult::Ok(Binary::default()))
                        }
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        })
                    }
                }
                WasmQuery::Smart { contract_addr, msg } => {
                    let Ok(addr) = api.addr_validate(contract_addr) else {
                        return SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        });
                    };
                    if addr == contract1 {
                        #[derive(Deserialize)]
                        struct MyMsg {}
                        let _msg: MyMsg = match from_json(msg) {
                            Ok(msg) => msg,
                            Err(err) => {
                                return SystemResult::Ok(ContractResult::Err(err.to_string()))
                            }
                        };
                        let response: Response = Response::new().set_data(b"good");
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        })
                    }
                }
                WasmQuery::ContractInfo { contract_addr } => {
                    let Ok(addr) = api.addr_validate(contract_addr) else {
                        return SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        });
                    };
                    if addr == contract1 {
                        let response = ContractInfoResponse {
                            code_id: 4,
                            creator: Addr::unchecked("lalala"),
                            admin: None,
                            pinned: false,
                            ibc_port: None,
                            ibc2_port: None,
                        };
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        })
                    }
                }
                #[cfg(feature = "cosmwasm_1_2")]
                WasmQuery::CodeInfo { code_id } => {
                    use crate::{Checksum, CodeInfoResponse};
                    let code_id = *code_id;
                    if code_id == 4 {
                        let response = CodeInfoResponse {
                            code_id,
                            creator: Addr::unchecked("lalala"),
                            checksum: Checksum::from_hex(
                                "84cf20810fd429caf58898c3210fcb71759a27becddae08dbde8668ea2f4725d",
                            )
                            .unwrap(),
                        };
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                    } else {
                        SystemResult::Err(SystemError::NoSuchCode { code_id })
                    }
                }
                #[cfg(all(feature = "cosmwasm_3_0", feature = "iterator"))]
                WasmQuery::RawRange {
                    contract_addr,
                    start,
                    end,
                    limit,
                    order,
                } => {
                    let Ok(addr) = api.addr_validate(contract_addr) else {
                        return SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        });
                    };
                    if addr == contract1 {
                        let mut data: Vec<_> = storage1
                            .range(
                                start.as_ref().map(Binary::as_slice),
                                end.as_ref().map(Binary::as_slice),
                                *order,
                            )
                            .take(*limit as usize + 1) // take one more entry than limit
                            .map(|(key, value)| (Binary::new(key), Binary::new(value)))
                            .collect();

                        // if we have more than limit, there are more entries to fetch
                        let next_key = if data.len() > *limit as usize {
                            data.pop().map(|(key, _)| key)
                        } else {
                            None
                        };
                        let raw_range_response = crate::RawRangeResponse { data, next_key };

                        SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&raw_range_response).unwrap(),
                        ))
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.clone(),
                        })
                    }
                }
            }
        });

        let contract_addr = MockApi::default().addr_make("contract1");

        // WasmQuery::Raw
        let result = querier.query(&WasmQuery::Raw {
            contract_addr: contract_addr.clone().into(),
            key: b"the key".into(),
        });

        match result {
            SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(value, b"the value" as &[u8]),
            res => panic!("Unexpected result: {res:?}"),
        }
        let result = querier.query(&WasmQuery::Raw {
            contract_addr: contract_addr.clone().into(),
            key: b"other key".into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(value, b"" as &[u8]),
            res => panic!("Unexpected result: {res:?}"),
        }

        // WasmQuery::Smart
        let result = querier.query(&WasmQuery::Smart {
            contract_addr: contract_addr.clone().into(),
            msg: b"{}".into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(
                value,
                br#"{"messages":[],"attributes":[],"events":[],"data":"Z29vZA=="}"# as &[u8]
            ),
            res => panic!("Unexpected result: {res:?}"),
        }
        let result = querier.query(&WasmQuery::Smart {
            contract_addr: contract_addr.clone().into(),
            msg: b"a broken request".into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Err(err)) => {
                assert_eq!(err, "Error parsing into type cosmwasm_std::testing::mock::tests::wasm_querier_works::{{closure}}::MyMsg: expected value at line 1 column 1")
            }
            res => panic!("Unexpected result: {res:?}"),
        }

        // WasmQuery::ContractInfo
        let result = querier.query(&WasmQuery::ContractInfo {
            contract_addr: contract_addr.clone().into(),
        });
        match result {
            SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(
                value,
                br#"{"code_id":4,"creator":"lalala","admin":null,"pinned":false,"ibc_port":null,"ibc2_port":null}"#
                    as &[u8]
            ),
            res => panic!("Unexpected result: {res:?}"),
        }

        // WasmQuery::CodeInfo
        #[cfg(feature = "cosmwasm_1_2")]
        {
            let result = querier.query(&WasmQuery::CodeInfo { code_id: 4 });
            match result {
                SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(
                    value,
                    br#"{"code_id":4,"creator":"lalala","checksum":"84cf20810fd429caf58898c3210fcb71759a27becddae08dbde8668ea2f4725d"}"#
                ),
                res => panic!("Unexpected result: {res:?}"),
            }
        }

        #[cfg(all(feature = "cosmwasm_3_0", feature = "iterator"))]
        {
            let result = querier.query(&WasmQuery::RawRange {
                contract_addr: contract_addr.clone().into(),
                start: Some(Binary::from(b"the key")),
                end: Some(Binary::from(b"the keyasdf")),
                limit: 10,
                order: crate::Order::Ascending,
            });
            match result {
                SystemResult::Ok(ContractResult::Ok(value)) => assert_eq!(
                    value.as_slice(),
                    br#"{"data":[["dGhlIGtleQ==","dGhlIHZhbHVl"]],"next_key":null}"#
                ),
                res => panic!("Unexpected result: {res:?}"),
            }
        }
    }

    #[test]
    fn making_an_address_works() {
        let mock_api = MockApi::default();

        assert_eq!(
            mock_api.addr_make("creator").to_string(),
            "cosmwasm1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqs8s7vcp",
        );

        assert_eq!(
            mock_api.addr_make("").to_string(),
            "cosmwasm1uwcvgs5clswpfxhm7nyfjmaeysn6us0yvjdexn9yjkv3k7zjhp2sly4xh9",
        );

        let mock_api = MockApi::default().with_prefix("juno");
        assert_eq!(
            mock_api.addr_make("creator").to_string(),
            "juno1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqsksmtyp",
        );
    }

    #[test]
    fn colon_in_prefix_is_valid() {
        let mock_api = MockApi::default().with_prefix("did:com:");
        let addr = mock_api
            .addr_validate("did:com:1jkf0kmeyefvyzpwf56m7sne2000ay53r6upttu")
            .unwrap();

        assert_eq!(
            addr.as_str(),
            "did:com:1jkf0kmeyefvyzpwf56m7sne2000ay53r6upttu"
        );
    }

    #[test]
    #[should_panic(
        expected = "Generating address failed with reason: hrp is empty, must have at least 1 character"
    )]
    fn making_an_address_with_empty_prefix_should_panic() {
        MockApi::default().with_prefix("").addr_make("creator");
    }

    #[test]
    #[cfg(feature = "cosmwasm_1_3")]
    fn distribution_querier_new_works() {
        let addresses = [
            ("addr0000".to_string(), "addr0001".to_string()),
            ("addr0002".to_string(), "addr0001".to_string()),
        ];
        let btree_map = BTreeMap::from(addresses.clone());

        // should still work with HashMap
        let hashmap = std::collections::HashMap::from(addresses.clone());
        let querier = DistributionQuerier::new(hashmap);
        assert_eq!(querier.withdraw_addresses, btree_map);

        // should work with BTreeMap
        let querier = DistributionQuerier::new(btree_map.clone());
        assert_eq!(querier.withdraw_addresses, btree_map);

        // should work with array
        let querier = DistributionQuerier::new(addresses);
        assert_eq!(querier.withdraw_addresses, btree_map);
    }

    #[test]
    fn instantiate2_address_can_be_humanized() {
        let mock_api = MockApi::default();

        let contract_addr = mock_api
            .addr_canonicalize(mock_api.addr_make("contract").as_str())
            .unwrap();
        let checksum =
            HexBinary::from_hex("9af782a3a1bcbcd22dbb6a45c751551d9af782a3a1bcbcd22dbb6a45c751551d")
                .unwrap();
        let salt = b"instance 1231";
        let canonical_addr = instantiate2_address(&checksum, &contract_addr, salt).unwrap();
        // we are not interested in the exact humanization, just that it works
        mock_api.addr_humanize(&canonical_addr).unwrap();
    }
}
