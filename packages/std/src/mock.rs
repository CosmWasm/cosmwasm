use snafu::ResultExt;
use std::collections::HashMap;

use crate::api::{ApiError, ApiSystemError};
use crate::coins::Coin;
use crate::encoding::Binary;
use crate::errors::{contract_err, StdResult, Utf8StringErr};
use crate::query::{AllBalanceResponse, BalanceResponse, BankQuery, QueryRequest};
use crate::serde::to_vec;
use crate::storage::MemoryStorage;
use crate::traits::{Api, Extern, Querier};
use crate::types::{BlockInfo, CanonicalAddr, ContractInfo, Env, HumanAddr, MessageInfo};

static CONTRACT_ADDR: &str = "cosmos2contract";

/// All external requirements that can be injected for unit tests.
/// It sets the given balance for the contract itself, nothing else
pub fn mock_dependencies(
    canonical_length: usize,
    contract_balance: &[Coin],
) -> Extern<MockStorage, MockApi, MockQuerier> {
    let contract_addr = HumanAddr::from(CONTRACT_ADDR);
    Extern {
        storage: MockStorage::new(),
        api: MockApi::new(canonical_length),
        querier: MockQuerier::new(&[(&contract_addr, contract_balance)]),
    }
}

/// Initializes the querier along with the mock_dependencies.
/// Sets all balances provided (yoy must explicitly set contract balance if desired)
pub fn mock_dependencies_with_balances(
    canonical_length: usize,
    balances: &[(&HumanAddr, &[Coin])],
) -> Extern<MockStorage, MockApi, MockQuerier> {
    Extern {
        storage: MockStorage::new(),
        api: MockApi::new(canonical_length),
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
    canonical_length: usize,
}

impl MockApi {
    pub fn new(canonical_length: usize) -> Self {
        MockApi { canonical_length }
    }
}

impl Default for MockApi {
    fn default() -> Self {
        Self::new(20)
    }
}

impl Api for MockApi {
    fn canonical_address(&self, human: &HumanAddr) -> StdResult<CanonicalAddr> {
        // Dummy input validation. This is more sophisticated for formats like bech32, where format and checksum are validated.
        if human.len() < 3 {
            return contract_err("Invalid input: human address too short");
        }
        if human.len() > self.canonical_length {
            return contract_err("Invalid input: human address too long");
        }

        let mut out = Vec::from(human.as_str());
        let append = self.canonical_length - out.len();
        if append > 0 {
            out.extend(vec![0u8; append]);
        }
        Ok(CanonicalAddr(Binary(out)))
    }

    fn human_address(&self, canonical: &CanonicalAddr) -> StdResult<HumanAddr> {
        if canonical.len() != self.canonical_length {
            return contract_err("Invalid input: canonical address length not correct");
        }

        // remove trailing 0's (TODO: fix this - but fine for first tests)
        let trimmed: Vec<u8> = canonical
            .as_slice()
            .iter()
            .cloned()
            .filter(|&x| x != 0)
            .collect();
        // convert to utf8
        let human = String::from_utf8(trimmed).context(Utf8StringErr {})?;
        Ok(HumanAddr(human))
    }
}

// just set signer, sent funds, and balance - rest given defaults
// this is intended for use in testcode only
pub fn mock_env<T: Api, U: Into<HumanAddr>>(api: &T, signer: U, sent: &[Coin]) -> Env {
    let signer = signer.into();
    Env {
        block: BlockInfo {
            height: 12_345,
            time: 1_571_797_419,
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        message: MessageInfo {
            signer: api.canonical_address(&signer).unwrap(),
            sent_funds: sent.to_vec(),
        },
        contract: ContractInfo {
            address: api
                .canonical_address(&HumanAddr::from(CONTRACT_ADDR))
                .unwrap(),
        },
    }
}

/// MockQuerier holds an immutable table of bank balances
/// TODO: also allow querying contracts
#[derive(Clone)]
pub struct MockQuerier {
    balances: HashMap<HumanAddr, Vec<Coin>>,
}

impl MockQuerier {
    pub fn new(balances: &[(&HumanAddr, &[Coin])]) -> Self {
        let mut map = HashMap::new();
        for (addr, coins) in balances.iter() {
            map.insert(HumanAddr::from(addr), coins.to_vec());
        }
        MockQuerier { balances: map }
    }
}

impl Querier for MockQuerier {
    fn query(&self, request: &QueryRequest) -> Result<Result<Binary, ApiError>, ApiSystemError> {
        match request {
            QueryRequest::Bank(bank) => match bank {
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
                    let api_res = to_vec(&bank_res).map(Binary).map_err(|e| e.into());
                    Ok(api_res)
                }
                BankQuery::AllBalances { address } => {
                    // proper error on not found, serialize result on found
                    let bank_res = AllBalanceResponse {
                        amount: self.balances.get(address).cloned().unwrap_or_default(),
                    };
                    let api_res = to_vec(&bank_res).map(Binary).map_err(|e| e.into());
                    Ok(api_res)
                }
            },
            QueryRequest::Contract { contract_addr, .. } => Err(ApiSystemError::NoSuchContract {
                addr: contract_addr.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::coins;

    #[test]
    fn mock_env_arguments() {
        let name = HumanAddr("my name".to_string());
        let api = MockApi::new(20);

        // make sure we can generate with &str, &HumanAddr, and HumanAddr
        let a = mock_env(&api, "my name", &coins(100, "atom"));
        let b = mock_env(&api, &name, &coins(100, "atom"));
        let c = mock_env(&api, name, &coins(100, "atom"));

        // and the results are the same
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn flip_addresses() {
        let api = MockApi::new(20);
        let human = HumanAddr("shorty".to_string());
        let canon = api.canonical_address(&human).unwrap();
        assert_eq!(canon.len(), 20);
        assert_eq!(&canon.as_slice()[0..6], human.as_str().as_bytes());
        assert_eq!(&canon.as_slice()[6..], &[0u8; 14]);

        let recovered = api.human_address(&canon).unwrap();
        assert_eq!(human, recovered);
    }

    #[test]
    #[should_panic(expected = "length not correct")]
    fn human_address_input_length() {
        let api = MockApi::new(10);
        let input = CanonicalAddr(Binary(vec![61; 11]));
        api.human_address(&input).unwrap();
    }

    #[test]
    #[should_panic(expected = "address too short")]
    fn canonical_address_min_input_length() {
        let api = MockApi::new(10);
        let human = HumanAddr("1".to_string());
        let _ = api.canonical_address(&human).unwrap();
    }

    #[test]
    #[should_panic(expected = "address too long")]
    fn canonical_address_max_input_length() {
        let api = MockApi::new(10);
        let human = HumanAddr("longer-than-10".to_string());
        let _ = api.canonical_address(&human).unwrap();
    }
}
