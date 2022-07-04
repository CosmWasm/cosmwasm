use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::ops::Deref;

use crate::addresses::{Addr, CanonicalAddr};
use crate::binary::Binary;
use crate::coins::Coin;
use crate::errors::{RecoverPubkeyError, SigningError, StdError, StdResult, VerificationError};
#[cfg(feature = "iterator")]
use crate::iterator::{Order, Record};
use crate::query::{
    AllBalanceResponse, BalanceResponse, BankQuery, CustomQuery, QueryRequest, WasmQuery,
};
#[cfg(feature = "staking")]
use crate::query::{
    AllDelegationsResponse, AllValidatorsResponse, BondedDenomResponse, Delegation,
    DelegationResponse, FullDelegation, StakingQuery, Validator, ValidatorResponse,
};
use crate::results::{ContractResult, Empty, SystemResult};
use crate::serde::{from_binary, to_binary, to_vec};

/// Storage provides read and write access to a persistent storage.
/// If you only want to provide read access, provide `&Storage`
pub trait Storage {
    /// Returns None when key does not exist.
    /// Returns Some(Vec<u8>) when key exists.
    ///
    /// Note: Support for differentiating between a non-existent key and a key with empty value
    /// is not great yet and might not be possible in all backends. But we're trying to get there.
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;

    #[cfg(feature = "iterator")]
    /// Allows iteration over a set of key/value pairs, either forwards or backwards.
    ///
    /// The bound `start` is inclusive and `end` is exclusive.
    ///
    /// If `start` is lexicographically greater than or equal to `end`, an empty range is described, mo matter of the order.
    fn range<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Record> + 'a>;

    fn set(&mut self, key: &[u8], value: &[u8]);

    /// Removes a database entry at `key`.
    ///
    /// The current interface does not allow to differentiate between a key that existed
    /// before and one that didn't exist. See https://github.com/CosmWasm/cosmwasm/issues/290
    fn remove(&mut self, key: &[u8]);
}

/// Api are callbacks to system functions implemented outside of the wasm modules.
/// Currently it just supports address conversion but we could add eg. crypto functions here.
///
/// This is a trait to allow mocks in the test code. Its members have a read-only
/// reference to the Api instance to allow accessing configuration.
/// Implementations must not have mutable state, such that an instance can freely
/// be copied and shared between threads without affecting the behaviour.
/// Given an Api instance, all members should return the same value when called with the same
/// arguments. In particular this means the result must not depend in the state of the chain.
/// If you need to access chaim state, you probably want to use the Querier.
/// Side effects (such as logging) are allowed.
///
/// We can use feature flags to opt-in to non-essential methods
/// for backwards compatibility in systems that don't have them all.
pub trait Api {
    /// Takes a human readable address and validates if it is valid.
    /// If it the validation succeeds, a `Addr` containing the same data as the input is returned.
    ///
    /// This validation checks two things:
    /// 1. The address is valid in the sense that it can be converted to a canonical representation by the backend.
    /// 2. The address is normalized, i.e. `humanize(canonicalize(input)) == input`.
    ///
    /// Check #2 is typically needed for upper/lower case representations of the same
    /// address that are both valid according to #1. This way we ensure uniqueness
    /// of the human readable address. Clients should perform the normalization before sending
    /// the addresses to the CosmWasm stack. But please note that the definition of normalized
    /// depends on the backend.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{Api, Addr};
    /// # use cosmwasm_std::testing::MockApi;
    /// # let api = MockApi::default();
    /// let input = "what-users-provide";
    /// let validated: Addr = api.addr_validate(input).unwrap();
    /// assert_eq!(validated, input);
    /// ```
    fn addr_validate(&self, human: &str) -> StdResult<Addr>;

    /// Takes a human readable address and returns a canonical binary representation of it.
    /// This can be used when a compact fixed length representation is needed.
    fn addr_canonicalize(&self, human: &str) -> StdResult<CanonicalAddr>;

    /// Takes a canonical address and returns a human readble address.
    /// This is the inverse of [`addr_canonicalize`].
    ///
    /// [`addr_canonicalize`]: Api::addr_canonicalize
    fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr>;

    /// ECDSA secp256k1 signature verification.
    ///
    /// This function verifies message hashes (hashed unsing SHA-256) against a signature,
    /// with the public key of the signer, using the secp256k1 elliptic curve digital signature
    /// parametrization / algorithm.
    ///
    /// The signature and public key are in "Cosmos" format:
    /// - signature:  Serialized "compact" signature (64 bytes).
    /// - public key: [Serialized according to SEC 2](https://www.oreilly.com/library/view/programming-bitcoin/9781492031482/ch04.html)
    fn secp256k1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError>;

    /// Recovers a public key from a message hash and a signature.
    ///
    /// This is required when working with Ethereum where public keys
    /// are not stored on chain directly.
    ///
    /// `recovery_param` must be 0 or 1. The values 2 and 3 are unsupported by this implementation,
    /// which is the same restriction as Ethereum has (https://github.com/ethereum/go-ethereum/blob/v1.9.25/internal/ethapi/api.go#L466-L469).
    /// All other values are invalid.
    ///
    /// Returns the recovered pubkey in compressed form, which can be used
    /// in secp256k1_verify directly.
    fn secp256k1_recover_pubkey(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        recovery_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError>;

    /// EdDSA ed25519 signature verification.
    ///
    /// This function verifies messages against a signature, with the public key of the signer,
    /// using the ed25519 elliptic curve digital signature parametrization / algorithm.
    ///
    /// The maximum currently supported message length is 4096 bytes.
    /// The signature and public key are in [Tendermint](https://docs.tendermint.com/v0.32/spec/blockchain/encoding.html#public-key-cryptography)
    /// format:
    /// - signature: raw ED25519 signature (64 bytes).
    /// - public key: raw ED25519 public key (32 bytes).
    fn ed25519_verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError>;

    /// Performs batch Ed25519 signature verification.
    ///
    /// Batch verification asks whether all signatures in some set are valid, rather than asking whether
    /// each of them is valid. This allows sharing computations among all signature verifications,
    /// performing less work overall, at the cost of higher latency (the entire batch must complete),
    /// complexity of caller code (which must assemble a batch of signatures across work-items),
    /// and loss of the ability to easily pinpoint failing signatures.
    ///
    /// This batch verification implementation is adaptive, in the sense that it detects multiple
    /// signatures created with the same verification key, and automatically coalesces terms
    /// in the final verification equation.
    ///
    /// In the limiting case where all signatures in the batch are made with the same verification key,
    /// coalesced batch verification runs twice as fast as ordinary batch verification.
    ///
    /// Three Variants are suppported in the input for convenience:
    ///  - Equal number of messages, signatures, and public keys: Standard, generic functionality.
    ///  - One message, and an equal number of signatures and public keys: Multiple digital signature
    /// (multisig) verification of a single message.
    ///  - One public key, and an equal number of messages and signatures: Verification of multiple
    /// messages, all signed with the same private key.
    ///
    /// Any other variants of input vectors result in an error.
    ///
    /// Notes:
    ///  - The "one-message, with zero signatures and zero public keys" case, is considered the empty case.
    ///  - The "one-public key, with zero messages and zero signatures" case, is considered the empty case.
    ///  - The empty case (no messages, no signatures and no public keys) returns true.
    fn ed25519_batch_verify(
        &self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<bool, VerificationError>;

    /// Emits a debugging message that is handled depending on the environment (typically printed to console or ignored).
    /// Those messages are not persisted to chain.
    fn debug(&self, message: &str);

    /// ECDSA secp256k1 signing.
    ///
    /// This function signs a message with a private key using the secp256k1 elliptic curve digital signature parametrization / algorithm.
    ///
    /// - message: Arbitrary message.
    /// - private key: Raw secp256k1 private key (32 bytes)
    fn secp256k1_sign(&self, message: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SigningError>;

    /// EdDSA Ed25519 signing.
    ///
    /// This function signs a message with a private key using the ed25519 elliptic curve digital signature parametrization / algorithm.
    ///
    /// - message: Arbitrary message.
    /// - private key: Raw ED25519 private key (32 bytes)
    fn ed25519_sign(&self, message: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SigningError>;
}

/// A short-hand alias for the two-level query result (1. accessing the contract, 2. executing query in the contract)
pub type QuerierResult = SystemResult<ContractResult<Binary>>;

pub trait Querier {
    /// raw_query is all that must be implemented for the Querier.
    /// This allows us to pass through binary queries from one level to another without
    /// knowing the custom format, or we can decode it, with the knowledge of the allowed
    /// types. People using the querier probably want one of the simpler auto-generated
    /// helper methods
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult;
}

#[derive(Clone)]
pub struct QuerierWrapper<'a, C: CustomQuery = Empty> {
    querier: &'a dyn Querier,
    custom_query_type: PhantomData<C>,
}

// Use custom implementation on order to implement Copy in case `C` is not `Copy`.
// See "There is a small difference between the two: the derive strategy will also
// place a Copy bound on type parameters, which isn’t always desired."
// https://doc.rust-lang.org/std/marker/trait.Copy.html
impl<'a, C: CustomQuery> Copy for QuerierWrapper<'a, C> {}

/// This allows us to use self.raw_query to access the querier.
/// It also allows external callers to access the querier easily.
impl<'a, C: CustomQuery> Deref for QuerierWrapper<'a, C> {
    type Target = dyn Querier + 'a;

    fn deref(&self) -> &Self::Target {
        self.querier
    }
}

impl<'a, C: CustomQuery> QuerierWrapper<'a, C> {
    pub fn new(querier: &'a dyn Querier) -> Self {
        QuerierWrapper {
            querier,
            custom_query_type: PhantomData,
        }
    }

    /// Makes the query and parses the response.
    ///
    /// Any error (System Error, Error or called contract, or Parse Error) are flattened into
    /// one level. Only use this if you don't need to check the SystemError
    /// eg. If you don't differentiate between contract missing and contract returned error
    pub fn query<U: DeserializeOwned>(&self, request: &QueryRequest<C>) -> StdResult<U> {
        let raw = to_vec(request).map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
        })?;
        match self.raw_query(&raw) {
            SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
                "Querier system error: {}",
                system_err
            ))),
            SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(
                format!("Querier contract error: {}", contract_err),
            )),
            SystemResult::Ok(ContractResult::Ok(value)) => from_binary(&value),
        }
    }

    pub fn query_balance(
        &self,
        address: impl Into<String>,
        denom: impl Into<String>,
    ) -> StdResult<Coin> {
        let request = BankQuery::Balance {
            address: address.into(),
            denom: denom.into(),
        }
        .into();
        let res: BalanceResponse = self.query(&request)?;
        Ok(res.amount)
    }

    pub fn query_all_balances(&self, address: impl Into<String>) -> StdResult<Vec<Coin>> {
        let request = BankQuery::AllBalances {
            address: address.into(),
        }
        .into();
        let res: AllBalanceResponse = self.query(&request)?;
        Ok(res.amount)
    }

    // this queries another wasm contract. You should know a priori the proper types for T and U
    // (response and request) based on the contract API
    pub fn query_wasm_smart<T: DeserializeOwned>(
        &self,
        code_hash: impl Into<String>,
        contract_addr: impl Into<String>,
        msg: &impl Serialize,
    ) -> StdResult<T> {
        let request = WasmQuery::Smart {
            contract_addr: contract_addr.into(),
            code_hash: code_hash.into(),
            msg: to_binary(msg)?,
        }
        .into();
        self.query(&request)
    }

    #[cfg(feature = "staking")]
    pub fn query_all_validators(&self) -> StdResult<Vec<Validator>> {
        let request = StakingQuery::AllValidators {}.into();
        let res: AllValidatorsResponse = self.query(&request)?;
        Ok(res.validators)
    }

    #[cfg(feature = "staking")]
    pub fn query_validator(&self, address: impl Into<String>) -> StdResult<Option<Validator>> {
        let request = StakingQuery::Validator {
            address: address.into(),
        }
        .into();
        let res: ValidatorResponse = self.query(&request)?;
        Ok(res.validator)
    }

    #[cfg(feature = "staking")]
    pub fn query_bonded_denom(&self) -> StdResult<String> {
        let request = StakingQuery::BondedDenom {}.into();
        let res: BondedDenomResponse = self.query(&request)?;
        Ok(res.denom)
    }

    #[cfg(feature = "staking")]
    pub fn query_all_delegations(
        &self,
        delegator: impl Into<String>,
    ) -> StdResult<Vec<Delegation>> {
        let request = StakingQuery::AllDelegations {
            delegator: delegator.into(),
        }
        .into();
        let res: AllDelegationsResponse = self.query(&request)?;
        Ok(res.delegations)
    }

    #[cfg(feature = "staking")]
    pub fn query_delegation(
        &self,
        delegator: impl Into<String>,
        validator: impl Into<String>,
    ) -> StdResult<Option<FullDelegation>> {
        let request = StakingQuery::Delegation {
            delegator: delegator.into(),
            validator: validator.into(),
        }
        .into();
        let res: DelegationResponse = self.query(&request)?;
        Ok(res.delegation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockQuerier;
    use crate::{coins, from_slice, Uint128};

    // this is a simple demo helper to prove we can use it
    fn demo_helper(_querier: &dyn Querier) -> u64 {
        2
    }

    // this just needs to compile to prove we can use it
    #[test]
    fn use_querier_wrapper_as_querier() {
        let querier: MockQuerier<Empty> = MockQuerier::new(&[]);
        let wrapper = QuerierWrapper::<Empty>::new(&querier);

        // call with deref shortcut
        let res = demo_helper(&*wrapper);
        assert_eq!(2, res);

        // call with explicit deref
        let res = demo_helper(wrapper.deref());
        assert_eq!(2, res);
    }

    #[test]
    fn auto_deref_raw_query() {
        let acct = String::from("foobar");
        let querier: MockQuerier<Empty> = MockQuerier::new(&[(&acct, &coins(5, "BTC"))]);
        let wrapper = QuerierWrapper::<Empty>::new(&querier);
        let query = QueryRequest::<Empty>::Bank(BankQuery::Balance {
            address: acct,
            denom: "BTC".to_string(),
        });

        let raw = wrapper
            .raw_query(&to_vec(&query).unwrap())
            .unwrap()
            .unwrap();
        let balance: BalanceResponse = from_slice(&raw).unwrap();
        assert_eq!(balance.amount.amount, Uint128::new(5));
    }
}
