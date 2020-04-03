use snafu::ResultExt;
use std::collections::HashMap;

use crate::api::{ApiError, ApiSystemError};
use crate::encoding::Binary;
use crate::errors::{ContractErr, Result, Utf8StringErr};
use crate::query::{BalanceResponse, QueryRequest};
use crate::serde::to_vec;
use crate::storage::MemoryStorage;
use crate::traits::{Api, Extern, Querier};
use crate::types::{BlockInfo, CanonicalAddr, Coin, ContractInfo, Env, HumanAddr, MessageInfo};

/// All external requirements that can be injected for unit tests
pub fn mock_dependencies(canonical_length: usize) -> Extern<MockStorage, MockApi, MockQuerier> {
    Extern {
        storage: MockStorage::new(),
        api: MockApi::new(canonical_length),
        querier: MockQuerier::new(&[]),
    }
}

// This initializes the querier along with the mock_dependencies
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
    fn canonical_address(&self, human: &HumanAddr) -> Result<CanonicalAddr> {
        if human.len() > self.canonical_length {
            return ContractErr {
                msg: "human encoding too long",
            }
            .fail();
        }
        let mut out = Vec::from(human.as_str());
        let append = self.canonical_length - out.len();
        if append > 0 {
            out.extend(vec![0u8; append]);
        }
        Ok(CanonicalAddr(Binary(out)))
    }

    fn human_address(&self, canonical: &CanonicalAddr) -> Result<HumanAddr> {
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
pub fn mock_env<T: Api, U: Into<HumanAddr>>(
    api: &T,
    signer: U,
    sent: &[Coin],
    balance: &[Coin],
) -> Env {
    let signer = signer.into();
    Env {
        block: BlockInfo {
            height: 12_345,
            time: 1_571_797_419,
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        message: MessageInfo {
            signer: api.canonical_address(&signer).unwrap(),
            sent_funds: if sent.is_empty() {
                None
            } else {
                Some(sent.to_vec())
            },
        },
        contract: ContractInfo {
            address: api
                .canonical_address(&HumanAddr("cosmos2contract".to_string()))
                .unwrap(),
            balance: if balance.is_empty() {
                None
            } else {
                Some(balance.to_vec())
            },
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
            QueryRequest::Balance { address } => {
                // proper error on not found, serialize result on found
                let bank_res = BalanceResponse {
                    amount: self.balances.get(address).cloned(),
                };
                let api_res = to_vec(&bank_res).map(Binary).map_err(|e| e.into());
                Ok(api_res)
            }
            QueryRequest::Contract { contract_addr, .. } => Err(ApiSystemError::NoSuchContract {
                addr: contract_addr.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::coin;

    #[test]
    fn mock_env_arguments() {
        let name = HumanAddr("my name".to_string());
        let api = MockApi::new(20);

        // make sure we can generate with &str, &HumanAddr, and HumanAddr
        let a = mock_env(&api, "my name", &[], &coin("100", "atom"));
        let b = mock_env(&api, &name, &[], &coin("100", "atom"));
        let c = mock_env(&api, name, &[], &coin("100", "atom"));

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
    #[should_panic]
    fn canonical_length_enforced() {
        let api = MockApi::new(10);
        let human = HumanAddr("longer-than-10".to_string());
        let _ = api.canonical_address(&human).unwrap();
    }
}
