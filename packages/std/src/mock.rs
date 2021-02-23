use serde::de::DeserializeOwned;
#[cfg(feature = "stargate")]
use serde::Serialize;
use std::collections::HashMap;

use crate::addresses::{CanonicalAddr, HumanAddr};
use crate::binary::Binary;
use crate::coins::Coin;
use crate::deps::OwnedDeps;
use crate::errors::{RecoverPubkeyError, StdError, StdResult, SystemError, VerificationError};
#[cfg(feature = "stargate")]
use crate::ibc::{IbcChannel, IbcEndpoint, IbcOrder, IbcPacket, IbcTimeoutBlock};
use crate::query::{
    AllBalanceResponse, AllDelegationsResponse, BalanceResponse, BankQuery, BondedDenomResponse,
    CustomQuery, DelegationResponse, FullDelegation, QueryRequest, StakingQuery, Validator,
    ValidatorsResponse, WasmQuery,
};
use crate::results::{ContractResult, Empty, SystemResult};
use crate::serde::{from_slice, to_binary};
use crate::storage::MemoryStorage;
use crate::traits::{Api, Querier, QuerierResult};
use crate::types::{BlockInfo, ContractInfo, Env, MessageInfo};

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

/// All external requirements that can be injected for unit tests.
/// It sets the given balance for the contract itself, nothing else
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::new(&[(&contract_addr, contract_balance)]),
    }
}

/// Initializes the querier along with the mock_dependencies.
/// Sets all balances provided (yoy must explicitly set contract balance if desired)
pub fn mock_dependencies_with_balances(
    balances: &[(&HumanAddr, &[Coin])],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::new(balances),
    }
}

// Use MemoryStorage implementation (which is valid in non-testcode)
// We can later make simplifications here if needed
pub type MockStorage = MemoryStorage;

// MockPrecompiles zero pads all human addresses to make them fit the canonical_length
// it trims off zeros for the reverse operation.
// not really smart, but allows us to see a difference (and consistent length for canonical adddresses)
#[derive(Copy, Clone)]
pub struct MockApi {
    /// Length of canonical addresses created with this API. Contracts should not make any assumtions
    /// what this value is.
    pub canonical_length: usize,
}

impl Default for MockApi {
    fn default() -> Self {
        MockApi {
            canonical_length: 24,
        }
    }
}

impl Api for MockApi {
    fn canonical_address(&self, human: &HumanAddr) -> StdResult<CanonicalAddr> {
        // Dummy input validation. This is more sophisticated for formats like bech32, where format and checksum are validated.
        if human.len() < 3 {
            return Err(StdError::generic_err(
                "Invalid input: human address too short",
            ));
        }
        if human.len() > self.canonical_length {
            return Err(StdError::generic_err(
                "Invalid input: human address too long",
            ));
        }

        let mut out = Vec::from(human.as_str());

        // pad to canonical length with NULL bytes
        out.resize(self.canonical_length, 0x00);
        // content-dependent rotate followed by shuffle to destroy
        // the most obvious structure (https://github.com/CosmWasm/cosmwasm/issues/552)
        let rotate_by = digit_sum(&out) % self.canonical_length;
        out.rotate_left(rotate_by);
        for _ in 0..18 {
            out = riffle_shuffle(&out);
        }
        Ok(out.into())
    }

    fn human_address(&self, canonical: &CanonicalAddr) -> StdResult<HumanAddr> {
        if canonical.len() != self.canonical_length {
            return Err(StdError::generic_err(
                "Invalid input: canonical address length not correct",
            ));
        }

        let mut tmp: Vec<u8> = canonical.clone().into();
        // Shuffle two more times which restored the original value (24 elements are back to original after 20 rounds)
        for _ in 0..2 {
            tmp = riffle_shuffle(&tmp);
        }
        // Rotate back
        let rotate_by = digit_sum(&tmp) % self.canonical_length;
        tmp.rotate_right(rotate_by);
        // Remove NULL bytes (i.e. the padding)
        let trimmed = tmp.into_iter().filter(|&x| x != 0x00).collect();
        // decode UTF-8 bytes into string
        let human = String::from_utf8(trimmed)?;
        Ok(human.into())
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
            messages,
            signatures,
            public_keys,
        )?)
    }

    fn debug(&self, message: &str) {
        println!("{}", message);
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
            time: 1_571_797_419,
            time_nanos: 879305533,
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        contract: ContractInfo {
            address: HumanAddr::from(MOCK_CONTRACT_ADDR),
        },
    }
}

/// Just set sender and funds for the message.
/// This is intended for use in test code only.
pub fn mock_info<U: Into<HumanAddr>>(sender: U, funds: &[Coin]) -> MessageInfo {
    MessageInfo {
        sender: sender.into(),
        funds: funds.to_vec(),
    }
}

#[cfg(feature = "stargate")]
/// Creates an IbcChannel for testing. You set a few key parameters for handshaking,
/// If you want to set more, use this as a default and mutate other fields
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
        counterparty_version: Some(version.to_string()),
        connection_id: "connection-2".to_string(),
    }
}

#[cfg(feature = "stargate")]
/// Creates a IbcPacket for testing ibc_packet_receive. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields
pub fn mock_ibc_packet_recv<T: Serialize>(my_channel_id: &str, data: &T) -> StdResult<IbcPacket> {
    Ok(IbcPacket {
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
        timeout_block: Some(IbcTimeoutBlock {
            revision: 1,
            height: 12345678,
        }),
        timeout_timestamp: None,
    })
}

#[cfg(feature = "stargate")]
/// Creates a IbcPacket for testing ibc_packet_{ack,timeout}. You set a few key parameters that are
/// often parsed. If you want to set more, use this as a default and mutate other fields.
/// The difference between mock_ibc_packet_recv is if `my_channel_id` is src or dest.
pub fn mock_ibc_packet_ack<T: Serialize>(my_channel_id: &str, data: &T) -> StdResult<IbcPacket> {
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
        timeout_block: Some(IbcTimeoutBlock {
            revision: 1,
            height: 432332552,
        }),
        timeout_timestamp: None,
    })
}

/// The same type as cosmwasm-std's QuerierResult, but easier to reuse in
/// cosmwasm-vm. It might diverge from QuerierResult at some point.
pub type MockQuerierCustomHandlerResult = SystemResult<ContractResult<Binary>>;

/// MockQuerier holds an immutable table of bank balances
/// TODO: also allow querying contracts
pub struct MockQuerier<C: DeserializeOwned = Empty> {
    bank: BankQuerier,
    staking: StakingQuerier,
    // placeholder to add support later
    wasm: NoWasmQuerier,
    /// A handler to handle custom queries. This is set to a dummy handler that
    /// always errors by default. Update it via `with_custom_handler`.
    ///
    /// Use box to avoid the need of another generic type
    custom_handler: Box<dyn for<'a> Fn(&'a C) -> MockQuerierCustomHandlerResult>,
}

impl<C: DeserializeOwned> MockQuerier<C> {
    pub fn new(balances: &[(&HumanAddr, &[Coin])]) -> Self {
        MockQuerier {
            bank: BankQuerier::new(balances),
            staking: StakingQuerier::default(),
            wasm: NoWasmQuerier {},
            // strange argument notation suggested as a workaround here: https://github.com/rust-lang/rust/issues/41078#issuecomment-294296365
            custom_handler: Box::from(|_: &_| -> MockQuerierCustomHandlerResult {
                SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "custom".to_string(),
                })
            }),
        }
    }

    // set a new balance for the given address and return the old balance
    pub fn update_balance<U: Into<HumanAddr>>(
        &mut self,
        addr: U,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        self.bank.balances.insert(addr.into(), balance)
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

    pub fn with_custom_handler<CH: 'static>(mut self, handler: CH) -> Self
    where
        CH: Fn(&C) -> MockQuerierCustomHandlerResult,
    {
        self.custom_handler = Box::from(handler);
        self
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

#[derive(Clone, Default)]
struct NoWasmQuerier {
    // FIXME: actually provide a way to call out
}

impl NoWasmQuerier {
    fn query(&self, request: &WasmQuery) -> QuerierResult {
        let addr = match request {
            WasmQuery::Smart { contract_addr, .. } => contract_addr,
            WasmQuery::Raw { contract_addr, .. } => contract_addr,
        }
        .clone();
        SystemResult::Err(SystemError::NoSuchContract { addr })
    }
}

#[derive(Clone, Default)]
pub struct BankQuerier {
    balances: HashMap<HumanAddr, Vec<Coin>>,
}

impl BankQuerier {
    pub fn new(balances: &[(&HumanAddr, &[Coin])]) -> Self {
        let mut map = HashMap::new();
        for (addr, coins) in balances.iter() {
            map.insert(HumanAddr::from(addr), coins.to_vec());
        }
        BankQuerier { balances: map }
    }

    pub fn query(&self, request: &BankQuery) -> QuerierResult {
        let contract_result: ContractResult<Binary> = match request {
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

#[derive(Clone, Default)]
pub struct StakingQuerier {
    denom: String,
    validators: Vec<Validator>,
    delegations: Vec<FullDelegation>,
}

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
            StakingQuery::Validators {} => {
                let res = ValidatorsResponse {
                    validators: self.validators.clone(),
                };
                to_binary(&res).into()
            }
            StakingQuery::AllDelegations { delegator } => {
                let delegations: Vec<_> = self
                    .delegations
                    .iter()
                    .filter(|d| &d.delegator == delegator)
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
                    .find(|d| &d.delegator == delegator && &d.validator == validator);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::Delegation;
    use crate::{coin, coins, from_binary, Decimal, HumanAddr};
    use hex_literal::hex;

    const SECP256K1_MSG_HASH_HEX: &str =
        "5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0";
    const SECP256K1_SIG_HEX: &str = "207082eb2c3dfa0b454e0906051270ba4074ac93760ba9e7110cd9471475111151eb0dbbc9920e72146fb564f99d039802bf6ef2561446eb126ef364d21ee9c4";
    const SECP256K1_PUBKEY_HEX: &str = "04051c1ee2190ecfb174bfe4f90763f2b4ff7517b70a2aec1876ebcfd644c4633fb03f3cfbd94b1f376e34592d9d41ccaf640bb751b00a1fadeb0c01157769eb73";

    const ED25519_MSG_HEX: &str = "72";
    const ED25519_SIG_HEX: &str = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00";
    const ED25519_PUBKEY_HEX: &str =
        "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";

    #[test]
    fn mock_info_arguments() {
        let name = HumanAddr("my name".to_string());

        // make sure we can generate with &str, &HumanAddr, and HumanAddr
        let a = mock_info("my name", &coins(100, "atom"));
        let b = mock_info(&name, &coins(100, "atom"));
        let c = mock_info(name, &coins(100, "atom"));

        // and the results are the same
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn canonicalize_and_humanize_restores_original() {
        let api = MockApi::default();

        let original = HumanAddr::from("shorty");
        let canonical = api.canonical_address(&original).unwrap();
        let recovered = api.human_address(&canonical).unwrap();
        assert_eq!(recovered, original);
    }

    #[test]
    #[should_panic(expected = "length not correct")]
    fn human_address_input_length() {
        let api = MockApi::default();
        let input = CanonicalAddr(Binary(vec![61; 11]));
        api.human_address(&input).unwrap();
    }

    #[test]
    #[should_panic(expected = "address too short")]
    fn canonical_address_min_input_length() {
        let api = MockApi::default();
        let human = HumanAddr("1".to_string());
        let _ = api.canonical_address(&human).unwrap();
    }

    #[test]
    #[should_panic(expected = "address too long")]
    fn canonical_address_max_input_length() {
        let api = MockApi::default();
        let human = HumanAddr::from("some-extremely-long-address-not-supported-by-this-api");
        let _ = api.canonical_address(&human).unwrap();
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
        assert_eq!(res.unwrap_err(), VerificationError::PublicKeyErr);
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
        let mut corrupted_hash = hash.clone();
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
            RecoverPubkeyError::HashErr => {}
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
        assert_eq!(res.unwrap_err(), VerificationError::PublicKeyErr);
    }

    // Basic "works" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
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

    // Basic "fails" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
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

    // Basic "errors" test. Exhaustive tests on VM's side (packages/vm/src/imports.rs)
    #[test]
    fn ed25519_batch_verify_errs() {
        let api = MockApi::default();

        let msg = hex::decode(ED25519_MSG_HEX).unwrap();
        let signature = hex::decode(ED25519_SIG_HEX).unwrap();
        let public_key: Vec<u8> = vec![0u8; 0];

        let msgs: Vec<&[u8]> = vec![&msg.as_slice()];
        let signatures: Vec<&[u8]> = vec![&signature.as_slice()];
        let public_keys: Vec<&[u8]> = vec![&public_key];

        let res = api.ed25519_batch_verify(&msgs, &signatures, &public_keys);
        assert_eq!(res.unwrap_err(), VerificationError::PublicKeyErr);
    }

    #[test]
    fn bank_querier_all_balances() {
        let addr = HumanAddr::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let bank = BankQuerier::new(&[(&addr, &balance)]);

        // all
        let all = bank
            .query(&BankQuery::AllBalances {
                address: addr.clone(),
            })
            .unwrap()
            .unwrap();
        let res: AllBalanceResponse = from_binary(&all).unwrap();
        assert_eq!(&res.amount, &balance);
    }

    #[test]
    fn bank_querier_one_balance() {
        let addr = HumanAddr::from("foobar");
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
                address: addr.clone(),
                denom: "MISS".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_binary(&miss).unwrap();
        assert_eq!(res.amount, coin(0, "MISS"));
    }

    #[test]
    fn bank_querier_missing_account() {
        let addr = HumanAddr::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let bank = BankQuerier::new(&[(&addr, &balance)]);

        // all balances on empty account is empty vec
        let all = bank
            .query(&BankQuery::AllBalances {
                address: HumanAddr::from("elsewhere"),
            })
            .unwrap()
            .unwrap();
        let res: AllBalanceResponse = from_binary(&all).unwrap();
        assert_eq!(res.amount, vec![]);

        // any denom on balances on empty account is empty coin
        let miss = bank
            .query(&BankQuery::Balance {
                address: HumanAddr::from("elsewhere"),
                denom: "ELF".to_string(),
            })
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_binary(&miss).unwrap();
        assert_eq!(res.amount, coin(0, "ELF"));
    }

    #[test]
    fn staking_querier_validators() {
        let val1 = Validator {
            address: HumanAddr::from("validator-one"),
            commission: Decimal::percent(1),
            max_commission: Decimal::percent(3),
            max_change_rate: Decimal::percent(1),
        };
        let val2 = Validator {
            address: HumanAddr::from("validator-two"),
            commission: Decimal::permille(15),
            max_commission: Decimal::permille(40),
            max_change_rate: Decimal::permille(5),
        };

        let staking = StakingQuerier::new("ustake", &[val1.clone(), val2.clone()], &[]);

        // one match
        let raw = staking
            .query(&StakingQuery::Validators {})
            .unwrap()
            .unwrap();
        let vals: ValidatorsResponse = from_binary(&raw).unwrap();
        assert_eq!(vals.validators, vec![val1, val2]);
    }

    // gets delegators from query or panic
    fn get_all_delegators(staking: &StakingQuerier, delegator: HumanAddr) -> Vec<Delegation> {
        let raw = staking
            .query(&StakingQuery::AllDelegations { delegator })
            .unwrap()
            .unwrap();
        let dels: AllDelegationsResponse = from_binary(&raw).unwrap();
        dels.delegations
    }

    // gets full delegators from query or panic
    fn get_delegator(
        staking: &StakingQuerier,
        delegator: HumanAddr,
        validator: HumanAddr,
    ) -> Option<FullDelegation> {
        let raw = staking
            .query(&StakingQuery::Delegation {
                delegator,
                validator,
            })
            .unwrap()
            .unwrap();
        let dels: DelegationResponse = from_binary(&raw).unwrap();
        dels.delegation
    }

    #[test]
    fn staking_querier_delegations() {
        let val1 = HumanAddr::from("validator-one");
        let val2 = HumanAddr::from("validator-two");

        let user_a = HumanAddr::from("investor");
        let user_b = HumanAddr::from("speculator");
        let user_c = HumanAddr::from("hodler");

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
        let dels = get_all_delegators(&staking, HumanAddr::from("no one"));
        assert_eq!(dels, vec![]);

        // filter a by validator (1 and 1)
        let dels = get_delegator(&staking, user_a.clone(), val1.clone());
        assert_eq!(dels, Some(del1a.clone()));
        let dels = get_delegator(&staking, user_a.clone(), val2.clone());
        assert_eq!(dels, Some(del2a.clone()));

        // filter b by validator (2 and 0)
        let dels = get_delegator(&staking, user_b.clone(), val1.clone());
        assert_eq!(dels, Some(del1b.clone()));
        let dels = get_delegator(&staking, user_b.clone(), val2.clone());
        assert_eq!(dels, None);

        // filter c by validator (0 and 1)
        let dels = get_delegator(&staking, user_c.clone(), val1.clone());
        assert_eq!(dels, None);
        let dels = get_delegator(&staking, user_c.clone(), val2.clone());
        assert_eq!(dels, Some(del2c.clone()));
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
