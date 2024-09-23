use bech32::primitives::decode::CheckedHrpstring;
use bech32::{encode, Bech32, Hrp};
use cosmwasm_std::{
    Addr, BlockInfo, Coin, ContractInfo, Env, MessageInfo, Timestamp, TransactionInfo,
};
use sha2::{Digest, Sha256};

use super::querier::MockQuerier;
use super::storage::MockStorage;
use crate::backend::unwrap_or_return_with_gas;
use crate::{Backend, BackendApi, BackendError, BackendResult, GasInfo};

pub const MOCK_CONTRACT_ADDR: &str =
    "cosmwasm1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922tscp8avs";

/// Default gas multiplier in wasmd.
/// See https://github.com/CosmWasm/wasmd/blob/v0.51.0/x/wasm/types/gas_register.go#L34
const WASMD_GAS_MULTIPLIER: u64 = 140_000;
/// See https://github.com/CosmWasm/wasmd/blob/v0.51.0/x/wasm/keeper/api.go#L27
const GAS_COST_HUMANIZE: u64 = 4 * WASMD_GAS_MULTIPLIER;
/// See https://github.com/CosmWasm/wasmd/blob/v0.51.0/x/wasm/keeper/api.go#L28
const GAS_COST_CANONICALIZE: u64 = 5 * WASMD_GAS_MULTIPLIER;

/// Default prefix used when creating Bech32 encoded address.
const BECH32_PREFIX: &str = "cosmwasm";

/// All external requirements that can be injected for unit tests.
/// It sets the given balance for the contract itself, nothing else
pub fn mock_backend(contract_balance: &[Coin]) -> Backend<MockApi, MockStorage, MockQuerier> {
    Backend {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]),
    }
}

/// Initializes the querier along with the mock_dependencies.
/// Sets all balances provided (you must explicitly set contract balance if desired)
pub fn mock_backend_with_balances(
    balances: &[(&str, &[Coin])],
) -> Backend<MockApi, MockStorage, MockQuerier> {
    Backend {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: MockQuerier::new(balances),
    }
}

/// Zero-pads all human addresses to make them fit the canonical_length and
/// trims off zeros for the reverse operation.
/// This is not really smart, but allows us to see a difference (and consistent length for canonical addresses).
#[derive(Copy, Clone)]
pub struct MockApi(MockApiImpl);

#[derive(Copy, Clone)]
enum MockApiImpl {
    /// With this variant, all calls to the API fail with BackendError::Unknown
    /// containing the given message
    Error(&'static str),
    /// This variant implements Bech32 addresses.
    Bech32 {
        /// Prefix used for creating addresses in Bech32 encoding.
        bech32_prefix: &'static str,
    },
}

impl MockApi {
    pub fn new_failing(backend_error: &'static str) -> Self {
        Self(MockApiImpl::Error(backend_error))
    }

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
    /// assert_eq!(addr.as_str(), "juno1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqsksmtyp");
    /// ```
    pub fn with_prefix(self, prefix: &'static str) -> Self {
        Self(MockApiImpl::Bech32 {
            bech32_prefix: prefix,
        })
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
    /// assert_eq!(addr.as_str(), "cosmwasm1h34lmpywh4upnjdg90cjf4j70aee6z8qqfspugamjp42e4q28kqs8s7vcp");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics when generating a valid address is not possible,
    /// especially when Bech32 prefix set in function [with_prefix](Self::with_prefix) is empty.
    ///
    pub fn addr_make(&self, input: &str) -> String {
        // handle error case
        let bech32_prefix = match self.0 {
            MockApiImpl::Error(e) => panic!("Generating address failed: {e}"),
            MockApiImpl::Bech32 { bech32_prefix } => bech32_prefix,
        };

        let digest = Sha256::digest(input);
        let bech32_prefix = Hrp::parse(bech32_prefix).expect("Invalid prefix");
        match encode::<Bech32>(bech32_prefix, &digest) {
            Ok(address) => address,
            Err(reason) => panic!("Generating address failed with reason: {reason}"),
        }
    }
}

impl Default for MockApi {
    fn default() -> Self {
        Self(MockApiImpl::Bech32 {
            bech32_prefix: BECH32_PREFIX,
        })
    }
}

impl BackendApi for MockApi {
    fn addr_validate(&self, input: &str) -> BackendResult<()> {
        let mut gas_total = GasInfo {
            cost: 0,
            externally_used: 0,
        };

        let (canonicalize_res, gas_info) = self.addr_canonicalize(input);
        gas_total += gas_info;
        let canonical = unwrap_or_return_with_gas!(canonicalize_res, gas_total);

        let (humanize_res, gas_info) = self.addr_humanize(&canonical);
        gas_total += gas_info;
        let normalized = unwrap_or_return_with_gas!(humanize_res, gas_total);
        if input != normalized.as_str() {
            return (
                Err(BackendError::user_err(
                    "Invalid input: address not normalized",
                )),
                gas_total,
            );
        }
        (Ok(()), gas_total)
    }

    fn addr_canonicalize(&self, input: &str) -> BackendResult<Vec<u8>> {
        let gas_total = GasInfo::with_cost(GAS_COST_CANONICALIZE);

        // handle error case
        let bech32_prefix = match self.0 {
            MockApiImpl::Error(e) => return (Err(BackendError::unknown(e)), gas_total),
            MockApiImpl::Bech32 { bech32_prefix } => bech32_prefix,
        };

        let hrp_str = unwrap_or_return_with_gas!(
            CheckedHrpstring::new::<Bech32>(input)
                .map_err(|_| BackendError::user_err("Error decoding bech32")),
            gas_total
        );

        if !hrp_str
            .hrp()
            .as_bytes()
            .eq_ignore_ascii_case(bech32_prefix.as_bytes())
        {
            return (
                Err(BackendError::user_err("Wrong bech32 prefix")),
                gas_total,
            );
        }

        let bytes: Vec<u8> = hrp_str.byte_iter().collect();
        unwrap_or_return_with_gas!(validate_length(&bytes), gas_total);
        (Ok(bytes), gas_total)
    }

    fn addr_humanize(&self, canonical: &[u8]) -> BackendResult<String> {
        let gas_total = GasInfo::with_cost(GAS_COST_HUMANIZE);

        // handle error case
        let bech32_prefix = match self.0 {
            MockApiImpl::Error(e) => return (Err(BackendError::unknown(e)), gas_total),
            MockApiImpl::Bech32 { bech32_prefix } => bech32_prefix,
        };

        unwrap_or_return_with_gas!(validate_length(canonical), gas_total);
        let bech32_prefix = unwrap_or_return_with_gas!(
            Hrp::parse(bech32_prefix).map_err(|_| BackendError::user_err("Invalid bech32 prefix")),
            gas_total
        );
        let result = encode::<Bech32>(bech32_prefix, canonical)
            .map_err(|_| BackendError::user_err("Invalid data to be encoded to bech32"));

        (result, gas_total)
    }
}

/// Does basic validation of the number of bytes in a canonical address
fn validate_length(bytes: &[u8]) -> Result<(), BackendError> {
    match bytes.len() {
        1..=255 => Ok(()),
        _ => Err(BackendError::user_err("Invalid canonical address length")),
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
/// # use cosmwasm_std::{Addr, BlockInfo, ContractInfo, Env, Timestamp, TransactionInfo};
/// use cosmwasm_vm::testing::mock_env;
///
/// let env = mock_env();
/// assert_eq!(env, Env {
///     block: BlockInfo {
///         height: 12_345,
///         time: Timestamp::from_nanos(1_571_797_419_879_305_533),
///         chain_id: "cosmos-testnet-14002".to_string(),
///     },
///     transaction: Some(TransactionInfo { index: 3 }),
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
/// use cosmwasm_vm::testing::mock_env;
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
    let contract_addr = MockApi::default().addr_make("cosmos2contract");
    Env {
        block: BlockInfo {
            height: 12_345,
            time: Timestamp::from_nanos(1_571_797_419_879_305_533),
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        transaction: Some(TransactionInfo { index: 3 }),
        contract: ContractInfo {
            address: Addr::unchecked(contract_addr),
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;

    #[test]
    fn mock_env_matches_mock_contract_addr() {
        let contract_address = mock_env().contract.address;
        assert_eq!(contract_address, Addr::unchecked(MOCK_CONTRACT_ADDR));
    }

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
    fn addr_canonicalize_works() {
        let api = MockApi::default().with_prefix("osmo");

        api.addr_canonicalize("osmo186kh7c0k0gh4ww0wh4jqc4yhzu7n7dhswe845d")
            .0
            .unwrap();

        // is case insensitive
        let data1 = api
            .addr_canonicalize("osmo186kh7c0k0gh4ww0wh4jqc4yhzu7n7dhswe845d")
            .0
            .unwrap();
        let data2 = api
            .addr_canonicalize("OSMO186KH7C0K0GH4WW0WH4JQC4YHZU7N7DHSWE845D")
            .0
            .unwrap();
        assert_eq!(data1, data2);
    }

    #[test]
    fn canonicalize_and_humanize_restores_original() {
        let api = MockApi::default().with_prefix("juno");

        // simple
        let original = api.addr_make("shorty");
        let canonical = api.addr_canonicalize(&original).0.unwrap();
        let (recovered, _gas_cost) = api.addr_humanize(&canonical);
        assert_eq!(recovered.unwrap(), original);

        // normalizes input
        let original = "JUNO1MEPRU9FUQ4E65856ARD6068MFSFRWPGEMD0C3R";
        let canonical = api.addr_canonicalize(original).0.unwrap();
        let recovered = api.addr_humanize(&canonical).0.unwrap();
        assert_eq!(recovered, original.to_lowercase());

        // Long input (Juno contract address)
        let original =
            String::from("juno1v82su97skv6ucfqvuvswe0t5fph7pfsrtraxf0x33d8ylj5qnrysdvkc95");
        let canonical = api.addr_canonicalize(&original).0.unwrap();
        let recovered = api.addr_humanize(&canonical).0.unwrap();
        assert_eq!(recovered, original);
    }

    #[test]
    fn addr_humanize_input_length() {
        let api = MockApi::default();
        let input = vec![61; 256]; // too long
        let (result, _gas_info) = api.addr_humanize(&input);
        match result.unwrap_err() {
            BackendError::UserErr { .. } => {}
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn addr_canonicalize_min_input_length() {
        let api = MockApi::default();

        // empty address should fail
        let empty = "cosmwasm1pj90vm";
        assert!(matches!(api
            .addr_canonicalize(empty)
            .0
            .unwrap_err(),
            BackendError::UserErr { msg } if msg.contains("address length")));
    }

    #[test]
    fn addr_canonicalize_max_input_length() {
        let api = MockApi::default();

        let too_long = "cosmwasm1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqehqqkz";

        assert!(matches!(api
            .addr_canonicalize(too_long)
            .0
            .unwrap_err(),
            BackendError::UserErr { msg } if msg.contains("address length")));
    }

    #[test]
    fn colon_in_prefix_is_valid() {
        let mock_api = MockApi::default().with_prefix("did:com:");
        let bytes = mock_api
            .addr_canonicalize("did:com:1jkf0kmeyefvyzpwf56m7sne2000ay53r6upttu")
            .0
            .unwrap();
        let humanized = mock_api.addr_humanize(&bytes).0.unwrap();

        assert_eq!(
            humanized.as_str(),
            "did:com:1jkf0kmeyefvyzpwf56m7sne2000ay53r6upttu"
        );
    }
}
