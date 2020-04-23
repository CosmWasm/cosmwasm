use snafu::ResultExt;
use std::collections::HashMap;

use crate::api::SystemError;
use crate::coins::Coin;
use crate::encoding::Binary;
use crate::errors::{contract_err, StdResult, Utf8StringErr};
use crate::query::{AllBalanceResponse, BalanceResponse, BankQuery, QueryRequest, WasmQuery};
use crate::serde::to_binary;
use crate::storage::MemoryStorage;
use crate::traits::{Api, Extern, Querier, QuerierResult};
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

/// Just set sender and sent funds for the message. The rest uses defaults.
/// The sender will be canonicalized internally to allow developers pasing in human readable senders.
/// This is intended for use in test code only.
pub fn mock_env<T: Api, U: Into<HumanAddr>>(api: &T, sender: U, sent: &[Coin]) -> Env {
    Env {
        block: BlockInfo {
            height: 12_345,
            time: 1_571_797_419,
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        message: MessageInfo {
            sender: api.canonical_address(&sender.into()).unwrap(),
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
    bank: BankQuerier,
    #[cfg(feature = "staking")]
    staking: staking::StakingQuerier,
}

impl MockQuerier {
    #[cfg(not(feature = "staking"))]
    pub fn new(balances: &[(&HumanAddr, &[Coin])]) -> Self {
        MockQuerier {
            bank: BankQuerier::new(balances),
        }
    }

    #[cfg(feature = "staking")]
    pub fn new(balances: &[(&HumanAddr, &[Coin])]) -> Self {
        MockQuerier {
            bank: BankQuerier::new(balances),
            staking: staking::StakingQuerier::new(&[], &[]),
        }
    }

    #[cfg(feature = "staking")]
    pub fn with_staking(
        &mut self,
        validators: &[crate::query::Validator],
        delegations: &[crate::query::Delegation],
    ) {
        self.staking = staking::StakingQuerier::new(validators, delegations);
    }
}

#[derive(Clone)]
struct BankQuerier {
    balances: HashMap<HumanAddr, Vec<Coin>>,
}

impl BankQuerier {
    fn new(balances: &[(&HumanAddr, &[Coin])]) -> Self {
        let mut map = HashMap::new();
        for (addr, coins) in balances.iter() {
            map.insert(HumanAddr::from(addr), coins.to_vec());
        }
        BankQuerier { balances: map }
    }

    fn query(&self, request: &BankQuery) -> QuerierResult {
        match request {
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
                Ok(to_binary(&bank_res).map_err(|e| e.into()))
            }
            BankQuery::AllBalances { address } => {
                // proper error on not found, serialize result on found
                let bank_res = AllBalanceResponse {
                    amount: self.balances.get(address).cloned().unwrap_or_default(),
                };
                Ok(to_binary(&bank_res).map_err(|e| e.into()))
            }
        }
    }
}

#[cfg(feature = "staking")]
mod staking {
    use crate::api::{ApiError, ApiSystemError};
    use crate::encoding::Binary;
    use crate::query::{
        Delegation, DelegationsResponse, StakingQuery, Validator, ValidatorsResponse,
    };
    use crate::to_binary;

    #[derive(Clone)]
    pub struct StakingQuerier {
        validators: Vec<Validator>,
        delegations: Vec<Delegation>,
    }

    impl StakingQuerier {
        pub fn new(validators: &[Validator], delegations: &[Delegation]) -> Self {
            StakingQuerier {
                validators: validators.to_vec(),
                delegations: delegations.to_vec(),
            }
        }

        pub fn query(
            &self,
            request: &StakingQuery,
        ) -> Result<Result<Binary, ApiError>, ApiSystemError> {
            match request {
                StakingQuery::Validators {} => {
                    let val_res = ValidatorsResponse {
                        validators: self.validators.clone(),
                    };
                    Ok(to_binary(&val_res).map_err(|e| e.into()))
                }
                StakingQuery::Delegations {
                    delegator,
                    validator,
                } => {
                    let matches = |d: &&Delegation| {
                        if let Some(val) = validator {
                            if val != &d.validator {
                                return false;
                            }
                        }
                        &d.delegator == delegator
                    };
                    let delegations: Vec<_> =
                        self.delegations.iter().filter(matches).cloned().collect();
                    let val_res = DelegationsResponse { delegations };
                    Ok(to_binary(&val_res).map_err(|e| e.into()))
                }
            }
        }
    }
}

impl Querier for MockQuerier {
    fn query(&self, request: &QueryRequest) -> QuerierResult {
        match request {
            QueryRequest::Bank(bank_query) => self.bank.query(bank_query),
            #[cfg(feature = "staking")]
            QueryRequest::Staking(staking_query) => self.staking.query(staking_query),
            QueryRequest::Wasm(msg) => {
                let addr = match msg {
                    WasmQuery::Smart { contract_addr, .. } => contract_addr,
                    WasmQuery::Raw { contract_addr, .. } => contract_addr,
                }
                .clone();
                Err(SystemError::NoSuchContract { addr })
            }
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
