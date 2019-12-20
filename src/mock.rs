use std::collections::HashMap;
use std::str::from_utf8;

use snafu::ResultExt;

use crate::errors::{ContractErr, Result, Utf8Err};
use crate::traits::{Api, Extern, ReadonlyStorage, Storage};
use crate::types::{BlockInfo, Coin, ContractInfo, MessageInfo, Params};

// dependencies are all external requirements that can be injected for unit tests
pub fn dependencies(canonical_length: usize) -> Extern<MockStorage, MockApi> {
    Extern {
        storage: MockStorage::new(),
        api: MockApi::new(canonical_length),
    }
}

#[derive(Clone)]
pub struct MockStorage {
    data: HashMap<Vec<u8>, Vec<u8>>,
}

impl MockStorage {
    pub fn new() -> Self {
        MockStorage {
            data: HashMap::new(),
        }
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadonlyStorage for MockStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }
}

impl Storage for MockStorage {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.data.insert(key.to_vec(), value.to_vec());
    }
}

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
    fn canonical_address(&self, human: &str) -> Result<Vec<u8>> {
        if human.len() > self.canonical_length {
            return ContractErr {
                msg: "human encoding too long",
            }
            .fail();
        }
        let mut out = human.as_bytes().to_vec();
        let append = self.canonical_length - out.len();
        if append > 0 {
            out.extend(vec![0u8; append]);
        }
        Ok(out)
    }

    fn human_address(&self, canonical: &[u8]) -> Result<String> {
        // remove trailing 0's (TODO: fix this - but fine for first tests)
        let trimmed: Vec<u8> = canonical.iter().cloned().filter(|&x| x != 0).collect();
        // convert to utf8
        let human = from_utf8(&trimmed).context(Utf8Err {})?;
        Ok(human.to_string())
    }
}

// just set signer, sent funds, and balance - rest given defaults
// this is intended for use in testcode only
pub fn mock_params<T: Api>(
    precompiles: &T,
    signer: &str,
    sent: &[Coin],
    balance: &[Coin],
) -> Params {
    Params {
        block: BlockInfo {
            height: 12_345,
            time: 1_571_797_419,
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        message: MessageInfo {
            signer: precompiles.canonical_address(signer).unwrap(),
            sent_funds: if sent.is_empty() {
                None
            } else {
                Some(sent.to_vec())
            },
        },
        contract: ContractInfo {
            address: precompiles.canonical_address("cosmos2contract").unwrap(),
            balance: if balance.is_empty() {
                None
            } else {
                Some(balance.to_vec())
            },
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_and_set() {
        let mut store = MockStorage::new();
        assert_eq!(None, store.get(b"foo"));
        store.set(b"foo", b"bar");
        assert_eq!(Some(b"bar".to_vec()), store.get(b"foo"));
        assert_eq!(None, store.get(b"food"));
    }

    #[test]
    fn flip_addresses() {
        let precompiles = MockApi::new(20);
        let human = "shorty";
        let canon = precompiles.canonical_address(&human).unwrap();
        assert_eq!(canon.len(), 20);
        assert_eq!(&canon[0..6], human.as_bytes());
        assert_eq!(&canon[6..], &[0u8; 14]);

        let recovered = precompiles.human_address(&canon).unwrap();
        assert_eq!(human, &recovered);
    }

    #[test]
    #[should_panic]
    fn canonical_length_enforced() {
        let precompiles = MockApi::new(10);
        let human = "longer-than-10";
        let _ = precompiles.canonical_address(&human).unwrap();
    }
}
