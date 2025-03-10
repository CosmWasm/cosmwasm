use core::marker::PhantomData;
use core::ops::Deref;
use downcast_rs::{impl_downcast, Downcast};
use serde::{de::DeserializeOwned, Serialize};

use crate::coin::Coin;
#[cfg(feature = "iterator")]
use crate::iterator::{Order, Record};
use crate::prelude::*;
#[cfg(feature = "cosmwasm_1_2")]
use crate::query::CodeInfoResponse;
#[cfg(feature = "cosmwasm_1_1")]
use crate::query::SupplyResponse;
use crate::query::{
    AllBalanceResponse, BalanceResponse, BankQuery, CustomQuery, QueryRequest, WasmQuery,
};
#[cfg(feature = "staking")]
use crate::query::{
    AllDelegationsResponse, AllValidatorsResponse, BondedDenomResponse, Delegation,
    DelegationResponse, FullDelegation, StakingQuery, Validator, ValidatorResponse,
};
#[cfg(feature = "cosmwasm_1_3")]
use crate::query::{
    AllDenomMetadataResponse, DelegatorWithdrawAddressResponse, DenomMetadataResponse,
    DistributionQuery,
};
use crate::results::{ContractResult, Empty, SystemResult};
use crate::ContractInfoResponse;
use crate::{from_json, to_json_binary, to_json_vec, Binary};
use crate::{Addr, CanonicalAddr};
#[cfg(feature = "cosmwasm_1_3")]
use crate::{DenomMetadata, PageRequest};
use crate::{RecoverPubkeyError, StdError, StdResult, VerificationError};

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum HashFunction {
    Sha256 = 0,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<HashFunction> for cosmwasm_crypto::HashFunction {
    fn from(value: HashFunction) -> Self {
        match value {
            HashFunction::Sha256 => cosmwasm_crypto::HashFunction::Sha256,
        }
    }
}

/// Storage provides read and write access to a persistent storage.
/// If you only want to provide read access, provide `&Storage`
pub trait Storage {
    /// Returns None when key does not exist.
    /// Returns Some(Vec<u8>) when key exists.
    ///
    /// Note: Support for differentiating between a non-existent key and a key with empty value
    /// is not great yet and might not be possible in all backends. But we're trying to get there.
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;

    /// Allows iteration over a set of key/value pairs, either forwards or backwards.
    ///
    /// The bound `start` is inclusive and `end` is exclusive.
    /// If `start` is lexicographically greater than or equal to `end`, an empty range is described, mo matter of the order.
    #[cfg(feature = "iterator")]
    #[allow(unused_variables)]
    fn range<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Record> + 'a> {
        // This default implementation is just to avoid breaking code when enabling the `iterator` feature.
        // Any actual `Storage` impl should override this method.
        unimplemented!("This storage does not support ranging. Make sure to override the `range` method in your `Storage` implementation.")
    }

    /// Allows iteration over a set of keys, either forwards or backwards.
    ///
    /// The bound `start` is inclusive and `end` is exclusive.
    /// If `start` is lexicographically greater than or equal to `end`, an empty range is described, mo matter of the order.
    ///
    /// The default implementation uses [`Storage::range`] and discards the values. More efficient
    /// implementations might be possible depending on the storage.
    #[cfg(feature = "iterator")]
    fn range_keys<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Vec<u8>> + 'a> {
        Box::new(self.range(start, end, order).map(|(k, _v)| k))
    }

    /// Allows iteration over a set of values, either forwards or backwards.
    ///
    /// The bound `start` is inclusive and `end` is exclusive.
    /// If `start` is lexicographically greater than or equal to `end`, an empty range is described, mo matter of the order.
    ///
    /// The default implementation uses [`Storage::range`] and discards the keys. More efficient implementations
    /// might be possible depending on the storage.
    #[cfg(feature = "iterator")]
    fn range_values<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Vec<u8>> + 'a> {
        Box::new(self.range(start, end, order).map(|(_k, v)| v))
    }

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
pub trait Api: Downcast {
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
    /// let api = MockApi::default().with_prefix("juno");
    /// let input = "juno1v82su97skv6ucfqvuvswe0t5fph7pfsrtraxf0x33d8ylj5qnrysdvkc95";
    /// let validated: Addr = api.addr_validate(input).unwrap();
    /// assert_eq!(validated.as_str(), input);
    /// ```
    fn addr_validate(&self, human: &str) -> StdResult<Addr>;

    /// Takes a human readable address and returns a canonical binary representation of it.
    /// This can be used when a compact representation is needed.
    ///
    /// Please note that the length of the resulting address is defined by the chain and
    /// can vary from address to address. On Cosmos chains 20 and 32 bytes are typically used.
    /// But that might change. So your contract should not make assumptions on the size.
    fn addr_canonicalize(&self, human: &str) -> StdResult<CanonicalAddr>;

    /// Takes a canonical address and returns a human readable address.
    /// This is the inverse of [`addr_canonicalize`].
    ///
    /// [`addr_canonicalize`]: Api::addr_canonicalize
    fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr>;

    fn secp256k1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError>;

    fn secp256k1_recover_pubkey(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        recovery_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError>;

    /// Add up points of the G1 subgroup on the BLS12-381 curve
    ///
    /// The length of `g1s` must be a multiple of 48 (each point is encoded in 48 bytes).
    #[allow(unused_variables)]
    fn bls12_381_aggregate_g1(&self, g1s: &[u8]) -> Result<[u8; 48], VerificationError> {
        // Support for BLS12-381 is added in 2.1, i.e. we can't add a compile time requirement for new function.
        // Any implementation of the Api trait which does not implement this function but tries to call it will
        // panic at runtime. We don't assume such cases exist.
        // See also https://doc.rust-lang.org/cargo/reference/semver.html#trait-new-default-item
        unimplemented!()
    }

    /// Add up points of the G2 subgroup on the BLS12-381 curve
    ///
    /// The length of `g2s` must be a multiple of 96 (each point is encoded in 96 bytes)
    #[allow(unused_variables)]
    fn bls12_381_aggregate_g2(&self, g2s: &[u8]) -> Result<[u8; 96], VerificationError> {
        // Support for BLS12-381 is added in 2.1, i.e. we can't add a compile time requirement for new function.
        // Any implementation of the Api trait which does not implement this function but tries to call it will
        // panic at runtime. We don't assume such cases exist.
        // See also https://doc.rust-lang.org/cargo/reference/semver.html#trait-new-default-item
        unimplemented!()
    }

    /// Checks the following pairing equality:
    ///
    /// e(p_1, q_1) × e(p_2, q_2) × … × e(p_n, q_n) = e(s, q)
    ///
    /// The argument `ps` contain the points p_1, ..., p_n ∈ G1 as a concatenation of 48 byte elements.
    /// The argument `qs` contain the points q_1, ..., q_n ∈ G2 as a concatenation of 96 byte elements.
    ///
    /// ## Examples
    ///
    /// A simple signature check with one pairing on the left hand side (e(p, q) = e(s, q)):
    ///
    /// ```
    /// # use cosmwasm_std::{Api, HashFunction, StdResult};
    /// pub fn verify(
    ///     api: &dyn Api,
    ///     g1_generator: &[u8],
    ///     signature: &[u8],
    ///     pubkey: &[u8],
    ///     msg: &[u8],
    ///     dst: &[u8],
    /// ) -> StdResult<bool> {
    ///     let msg_hashed = api.bls12_381_hash_to_g2(HashFunction::Sha256, msg, dst)?;
    ///     api.bls12_381_pairing_equality(g1_generator, signature, pubkey, &msg_hashed)
    ///         .map_err(Into::into)
    /// }
    /// ```
    #[allow(unused_variables)]
    fn bls12_381_pairing_equality(
        &self,
        ps: &[u8],
        qs: &[u8],
        r: &[u8],
        s: &[u8],
    ) -> Result<bool, VerificationError> {
        // Support for BLS12-381 is added in 2.1, i.e. we can't add a compile time requirement for new function.
        // Any implementation of the Api trait which does not implement this function but tries to call it will
        // panic at runtime. We don't assume such cases exist.
        // See also https://doc.rust-lang.org/cargo/reference/semver.html#trait-new-default-item
        unimplemented!()
    }

    /// Take some arbitrary data and hash it to a point on the G1 subgroup of the curve.
    ///
    /// The `dst` parameter should be a constant is actually something similar to the "context" parameter in key derivation functions.
    #[allow(unused_variables)]
    fn bls12_381_hash_to_g1(
        &self,
        hash_function: HashFunction,
        msg: &[u8],
        dst: &[u8],
    ) -> Result<[u8; 48], VerificationError> {
        // Support for BLS12-381 is added in 2.1, i.e. we can't add a compile time requirement for new function.
        // Any implementation of the Api trait which does not implement this function but tries to call it will
        // panic at runtime. We don't assume such cases exist.
        // See also https://doc.rust-lang.org/cargo/reference/semver.html#trait-new-default-item
        unimplemented!()
    }

    /// Take some arbitrary data and hash it to a point on the G2 subgroup of the curve.
    ///
    /// The `dst` parameter should be a constant is actually something similar to the "context" parameter in key derivation functions.
    #[allow(unused_variables)]
    fn bls12_381_hash_to_g2(
        &self,
        hash_function: HashFunction,
        msg: &[u8],
        dst: &[u8],
    ) -> Result<[u8; 96], VerificationError> {
        // Support for BLS12-381 is added in 2.1, i.e. we can't add a compile time requirement for new function.
        // Any implementation of the Api trait which does not implement this function but tries to call it will
        // panic at runtime. We don't assume such cases exist.
        // See also https://doc.rust-lang.org/cargo/reference/semver.html#trait-new-default-item
        unimplemented!()
    }

    #[allow(unused_variables)]
    fn secp256r1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        // Support for secp256r1 is added in 2.1, i.e. we can't add a compile time requirement for new function.
        // Any implementation of the Api trait which does not implement this function but tries to call it will
        // panic at runtime. We don't assume such cases exist.
        // See also https://doc.rust-lang.org/cargo/reference/semver.html#trait-new-default-item
        unimplemented!()
    }

    #[allow(unused_variables)]
    fn secp256r1_recover_pubkey(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        recovery_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError> {
        // Support for secp256r1 was added in 2.1, i.e. we can't add a compile time requirement for new function.
        // Any implementation of the Api trait which does not implement this function but tries to call it will
        // panic at runtime. We don't assume such cases exist.
        // See also https://doc.rust-lang.org/cargo/reference/semver.html#trait-new-default-item
        unimplemented!()
    }

    fn ed25519_verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError>;

    fn ed25519_batch_verify(
        &self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<bool, VerificationError>;

    /// Emits a debugging message that is handled depending on the environment (typically printed to console or ignored).
    /// Those messages are not persisted to chain.
    fn debug(&self, message: &str);
}
impl_downcast!(Api);

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
impl<C: CustomQuery> Copy for QuerierWrapper<'_, C> {}

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

    /// This allows to convert any `QuerierWrapper` into a `QuerierWrapper` generic
    /// over `Empty` custom query type.
    pub fn into_empty(self) -> QuerierWrapper<'a, Empty> {
        QuerierWrapper {
            querier: self.querier,
            custom_query_type: PhantomData,
        }
    }

    /// Makes the query and parses the response.
    ///
    /// Any error (System Error, Error or called contract, or Parse Error) are flattened into
    /// one level. Only use this if you don't need to check the SystemError
    /// eg. If you don't differentiate between contract missing and contract returned error
    pub fn query<U: DeserializeOwned>(&self, request: &QueryRequest<C>) -> StdResult<U> {
        self.query_raw(request).and_then(|raw| from_json(raw))
    }

    /// Internal helper to avoid code duplication.
    /// Performs a query and returns the binary result without deserializing it,
    /// wrapping any errors that may occur into `StdError`.
    fn query_raw(&self, request: &QueryRequest<C>) -> StdResult<Binary> {
        let raw = to_json_vec(request).map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {serialize_err}"))
        })?;
        match self.raw_query(&raw) {
            SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
                "Querier system error: {system_err}"
            ))),
            SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(
                format!("Querier contract error: {contract_err}"),
            )),
            SystemResult::Ok(ContractResult::Ok(value)) => Ok(value),
        }
    }

    #[cfg(feature = "cosmwasm_1_1")]
    pub fn query_supply(&self, denom: impl Into<String>) -> StdResult<Coin> {
        let request = BankQuery::Supply {
            denom: denom.into(),
        }
        .into();
        let res: SupplyResponse = self.query(&request)?;
        Ok(res.amount)
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

    #[deprecated]
    pub fn query_all_balances(&self, address: impl Into<String>) -> StdResult<Vec<Coin>> {
        #[allow(deprecated)]
        let request = BankQuery::AllBalances {
            address: address.into(),
        }
        .into();
        let res: AllBalanceResponse = self.query(&request)?;
        Ok(res.amount)
    }

    #[cfg(feature = "cosmwasm_1_3")]
    pub fn query_delegator_withdraw_address(
        &self,
        delegator: impl Into<String>,
    ) -> StdResult<Addr> {
        let request = DistributionQuery::DelegatorWithdrawAddress {
            delegator_address: delegator.into(),
        }
        .into();
        let res: DelegatorWithdrawAddressResponse = self.query(&request)?;
        Ok(res.withdraw_address)
    }

    #[cfg(feature = "cosmwasm_1_3")]
    pub fn query_denom_metadata(&self, denom: impl Into<String>) -> StdResult<DenomMetadata> {
        let request = BankQuery::DenomMetadata {
            denom: denom.into(),
        }
        .into();
        let res: DenomMetadataResponse = self.query(&request)?;
        Ok(res.metadata)
    }

    #[cfg(feature = "cosmwasm_1_3")]
    pub fn query_all_denom_metadata(
        &self,
        pagination: PageRequest,
    ) -> StdResult<AllDenomMetadataResponse> {
        let request = BankQuery::AllDenomMetadata {
            pagination: Some(pagination),
        }
        .into();
        self.query(&request)
    }

    #[cfg(feature = "cosmwasm_1_4")]
    pub fn query_delegation_rewards(
        &self,
        delegator: impl Into<String>,
        validator: impl Into<String>,
    ) -> StdResult<Vec<crate::DecCoin>> {
        use crate::DelegationRewardsResponse;

        let request = DistributionQuery::DelegationRewards {
            delegator_address: delegator.into(),
            validator_address: validator.into(),
        }
        .into();
        let DelegationRewardsResponse { rewards } = self.query(&request)?;

        Ok(rewards)
    }

    #[cfg(feature = "cosmwasm_1_4")]
    pub fn query_delegation_total_rewards(
        &self,
        delegator: impl Into<String>,
    ) -> StdResult<crate::DelegationTotalRewardsResponse> {
        let request = DistributionQuery::DelegationTotalRewards {
            delegator_address: delegator.into(),
        }
        .into();
        self.query(&request)
    }

    #[cfg(feature = "cosmwasm_1_4")]
    pub fn query_delegator_validators(
        &self,
        delegator: impl Into<String>,
    ) -> StdResult<Vec<String>> {
        use crate::DelegatorValidatorsResponse;

        let request = DistributionQuery::DelegatorValidators {
            delegator_address: delegator.into(),
        }
        .into();
        let res: DelegatorValidatorsResponse = self.query(&request)?;
        Ok(res.validators)
    }

    /// See [`GrpcQuery`](crate::GrpcQuery) for more information.
    #[cfg(feature = "cosmwasm_2_0")]
    pub fn query_grpc(&self, path: String, data: Binary) -> StdResult<Binary> {
        use crate::GrpcQuery;
        self.query_raw(&QueryRequest::Grpc(GrpcQuery { path, data }))
    }

    /// Queries another wasm contract. You should know a priori the proper types for T and U
    /// (response and request) based on the contract API
    pub fn query_wasm_smart<T: DeserializeOwned>(
        &self,
        contract_addr: impl Into<String>,
        msg: &impl Serialize,
    ) -> StdResult<T> {
        let request = WasmQuery::Smart {
            contract_addr: contract_addr.into(),
            msg: to_json_binary(msg)?,
        }
        .into();
        self.query(&request)
    }

    /// Queries the raw storage from another wasm contract.
    ///
    /// You must know the exact layout and are implementation dependent
    /// (not tied to an interface like query_wasm_smart).
    /// That said, if you are building a few contracts together, this is a much cheaper approach
    ///
    /// Similar return value to [`Storage::get`]. Returns `Some(val)` or `None` if the data is there.
    /// It only returns error on some runtime issue, not on any data cases.
    pub fn query_wasm_raw(
        &self,
        contract_addr: impl Into<String>,
        key: impl Into<Binary>,
    ) -> StdResult<Option<Vec<u8>>> {
        let request: QueryRequest<Empty> = WasmQuery::Raw {
            contract_addr: contract_addr.into(),
            key: key.into(),
        }
        .into();
        // we cannot use query, as it will try to parse the binary data, when we just want to return it,
        // so a bit of code copy here...
        let raw = to_json_vec(&request).map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {serialize_err}"))
        })?;
        match self.raw_query(&raw) {
            SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
                "Querier system error: {system_err}"
            ))),
            SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(
                format!("Querier contract error: {contract_err}"),
            )),
            SystemResult::Ok(ContractResult::Ok(value)) => {
                if value.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(value.into()))
                }
            }
        }
    }

    /// Given a contract address, query information about that contract.
    pub fn query_wasm_contract_info(
        &self,
        contract_addr: impl Into<String>,
    ) -> StdResult<ContractInfoResponse> {
        let request = WasmQuery::ContractInfo {
            contract_addr: contract_addr.into(),
        }
        .into();
        self.query(&request)
    }

    /// Given a code ID, query information about that code.
    #[cfg(feature = "cosmwasm_1_2")]
    pub fn query_wasm_code_info(&self, code_id: u64) -> StdResult<CodeInfoResponse> {
        let request = WasmQuery::CodeInfo { code_id }.into();
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
    use serde::Deserialize;

    use super::*;
    use crate::testing::MockQuerier;
    use crate::{coins, Uint128};

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
            .raw_query(&to_json_vec(&query).unwrap())
            .unwrap()
            .unwrap();
        let balance: BalanceResponse = from_json(raw).unwrap();
        assert_eq!(balance.amount.amount, Uint128::new(5));
    }

    #[cfg(feature = "cosmwasm_1_1")]
    #[test]
    fn bank_query_helpers_work() {
        use crate::coin;

        let querier: MockQuerier<Empty> = MockQuerier::new(&[
            ("foo", &[coin(123, "ELF"), coin(777, "FLY")]),
            ("bar", &[coin(321, "ELF")]),
        ]);
        let wrapper = QuerierWrapper::<Empty>::new(&querier);

        let supply = wrapper.query_supply("ELF").unwrap();
        assert_eq!(supply, coin(444, "ELF"));

        let balance = wrapper.query_balance("foo", "ELF").unwrap();
        assert_eq!(balance, coin(123, "ELF"));

        #[allow(deprecated)]
        let all_balances = wrapper.query_all_balances("foo").unwrap();
        assert_eq!(all_balances, vec![coin(123, "ELF"), coin(777, "FLY")]);
    }

    #[test]
    fn contract_info() {
        const ACCT: &str = "foobar";
        fn mock_resp() -> ContractInfoResponse {
            ContractInfoResponse {
                code_id: 0,
                creator: Addr::unchecked("creator"),
                admin: None,
                pinned: false,
                ibc_port: None,
                ibc2_port: None,
            }
        }

        let mut querier: MockQuerier<Empty> = MockQuerier::new(&[(ACCT, &coins(5, "BTC"))]);
        querier.update_wasm(|q| -> QuerierResult {
            if q == &(WasmQuery::ContractInfo {
                contract_addr: ACCT.to_string(),
            }) {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&mock_resp()).unwrap()))
            } else {
                SystemResult::Err(crate::SystemError::NoSuchContract {
                    addr: ACCT.to_string(),
                })
            }
        });
        let wrapper = QuerierWrapper::<Empty>::new(&querier);

        let contract_info = wrapper.query_wasm_contract_info(ACCT).unwrap();
        assert_eq!(contract_info, mock_resp());
    }

    #[test]
    fn contract_info_err() {
        const ACCT: &str = "foobar";
        fn mock_resp() -> ContractInfoResponse {
            ContractInfoResponse {
                code_id: 0,
                creator: Addr::unchecked("creator"),
                admin: None,
                pinned: false,
                ibc_port: None,
                ibc2_port: None,
            }
        }

        let mut querier: MockQuerier<Empty> = MockQuerier::new(&[(ACCT, &coins(5, "BTC"))]);
        querier.update_wasm(|q| -> QuerierResult {
            if q == &(WasmQuery::ContractInfo {
                contract_addr: ACCT.to_string(),
            }) {
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&mock_resp()).unwrap()))
            } else {
                SystemResult::Err(crate::SystemError::NoSuchContract {
                    addr: ACCT.to_string(),
                })
            }
        });
        let wrapper = QuerierWrapper::<Empty>::new(&querier);

        let err = wrapper.query_wasm_contract_info("unknown").unwrap_err();
        assert!(matches!(
            err,
            StdError::GenericErr {
                msg,
                ..
            } if msg == "Querier system error: No such contract: foobar"
        ));
    }

    #[test]
    fn querier_into_empty() {
        #[derive(Clone, Serialize, Deserialize)]
        struct MyQuery;
        impl CustomQuery for MyQuery {}

        let querier: MockQuerier<MyQuery> = MockQuerier::new(&[]);
        let wrapper = QuerierWrapper::<MyQuery>::new(&querier);

        let _: QuerierWrapper<Empty> = wrapper.into_empty();
    }
}
