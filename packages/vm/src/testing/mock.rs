use cosmwasm_std::testing::{digit_sum, riffle_shuffle};
use cosmwasm_std::{BlockInfo, CanonicalAddr, Coin, ContractInfo, Env, HumanAddr, MessageInfo};

use super::querier::MockQuerier;
use super::storage::MockStorage;
use crate::{Api, Backend, BackendError, BackendResult, GasInfo};

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";
const GAS_COST_HUMANIZE: u64 = 44;
const GAS_COST_CANONICALIZE: u64 = 55;

/// All external requirements that can be injected for unit tests.
/// It sets the given balance for the contract itself, nothing else
pub fn mock_backend(contract_balance: &[Coin]) -> Backend<MockStorage, MockApi, MockQuerier> {
    let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
    Backend {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::new(&[(&contract_addr, contract_balance)]),
    }
}

/// Initializes the querier along with the mock_dependencies.
/// Sets all balances provided (yoy must explicitly set contract balance if desired)
pub fn mock_backend_with_balances(
    balances: &[(&HumanAddr, &[Coin])],
) -> Backend<MockStorage, MockApi, MockQuerier> {
    Backend {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::new(balances),
    }
}

/// Zero-pads all human addresses to make them fit the canonical_length and
/// trims off zeros for the reverse operation.
/// This is not really smart, but allows us to see a difference (and consistent length for canonical adddresses).
#[derive(Copy, Clone)]
pub struct MockApi {
    /// Length of canonical addresses created with this API. Contracts should not make any assumtions
    /// what this value is.
    pub canonical_length: usize,
    /// When set, all calls to the API fail with BackendError::Unknown containing this message
    backend_error: Option<&'static str>,
}

impl MockApi {
    #[deprecated(
        since = "0.11.0",
        note = "The canonical length argument is unused. Use MockApi::default() instead."
    )]
    pub fn new(_canonical_length: usize) -> Self {
        MockApi::default()
    }

    pub fn new_failing(backend_error: &'static str) -> Self {
        MockApi {
            backend_error: Some(backend_error),
            ..MockApi::default()
        }
    }
}

impl Default for MockApi {
    fn default() -> Self {
        MockApi {
            canonical_length: 24,
            backend_error: None,
        }
    }
}

impl Api for MockApi {
    fn canonical_address(&self, human: &HumanAddr) -> BackendResult<CanonicalAddr> {
        let gas_info = GasInfo::with_cost(GAS_COST_CANONICALIZE);

        if let Some(backend_error) = self.backend_error {
            return (Err(BackendError::unknown(backend_error)), gas_info);
        }

        // Dummy input validation. This is more sophisticated for formats like bech32, where format and checksum are validated.
        if human.len() < 3 {
            return (
                Err(BackendError::user_err(
                    "Invalid input: human address too short",
                )),
                gas_info,
            );
        }
        if human.len() > self.canonical_length {
            return (
                Err(BackendError::user_err(
                    "Invalid input: human address too long",
                )),
                gas_info,
            );
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
        (Ok(out.into()), gas_info)
    }

    fn human_address(&self, canonical: &CanonicalAddr) -> BackendResult<HumanAddr> {
        let gas_info = GasInfo::with_cost(GAS_COST_HUMANIZE);

        if let Some(backend_error) = self.backend_error {
            return (Err(BackendError::unknown(backend_error)), gas_info);
        }

        if canonical.len() != self.canonical_length {
            return (
                Err(BackendError::user_err(
                    "Invalid input: canonical address length not correct",
                )),
                gas_info,
            );
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

        let result = match String::from_utf8(trimmed) {
            Ok(human) => Ok(HumanAddr(human)),
            Err(err) => Err(err.into()),
        };
        (result, gas_info)
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

/// Just set sender and sent funds for the message. The essential for
/// This is intended for use in test code only.
pub fn mock_info<U: Into<HumanAddr>>(sender: U, sent: &[Coin]) -> MessageInfo {
    MessageInfo {
        sender: sender.into(),
        sent_funds: sent.to_vec(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::BackendError;
    use cosmwasm_std::{coins, Binary};

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
        let canonical = api.canonical_address(&original).0.unwrap();
        let (recovered, _gas_cost) = api.human_address(&canonical);
        assert_eq!(recovered.unwrap(), original);
    }

    #[test]
    fn human_address_input_length() {
        let api = MockApi::default();
        let input = CanonicalAddr(Binary(vec![61; 11]));
        let (result, _gas_info) = api.human_address(&input);
        match result.unwrap_err() {
            BackendError::UserErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn canonical_address_min_input_length() {
        let api = MockApi::default();
        let human = HumanAddr::from("1");
        match api.canonical_address(&human).0.unwrap_err() {
            BackendError::UserErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn canonical_address_max_input_length() {
        let api = MockApi::default();
        let human = HumanAddr::from("longer-than-the-address-length-supported-by-this-api");
        match api.canonical_address(&human).0.unwrap_err() {
            BackendError::UserErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }
}
