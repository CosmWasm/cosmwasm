use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::encoding::Binary;

// Added Eq and Hash to allow this to be a key in a HashMap (MockQuerier)
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema, Hash)]
pub struct HumanAddr(pub String);

impl HumanAddr {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for HumanAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<&str> for HumanAddr {
    fn from(addr: &str) -> Self {
        HumanAddr(addr.to_string())
    }
}

impl From<&HumanAddr> for HumanAddr {
    fn from(addr: &HumanAddr) -> Self {
        HumanAddr(addr.0.to_string())
    }
}

impl From<&&HumanAddr> for HumanAddr {
    fn from(addr: &&HumanAddr) -> Self {
        HumanAddr(addr.0.to_string())
    }
}

impl From<String> for HumanAddr {
    fn from(addr: String) -> Self {
        HumanAddr(addr)
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct CanonicalAddr(pub Binary);

impl From<&[u8]> for CanonicalAddr {
    fn from(source: &[u8]) -> Self {
        Self(source.into())
    }
}

impl From<Vec<u8>> for CanonicalAddr {
    fn from(source: Vec<u8>) -> Self {
        Self(source.into())
    }
}

impl CanonicalAddr {
    pub fn as_slice(&self) -> &[u8] {
        &self.0.as_slice()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for CanonicalAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Test HumanAddr as_str() for each HumanAddr::from input type
    #[test]
    fn human_addr_as_str() {
        // literal string
        let human_addr_from_literal_string = HumanAddr::from("literal-string");
        assert_eq!("literal-string", human_addr_from_literal_string.as_str());

        // String
        let addr = String::from("Hello, world!");
        let human_addr_from_string = HumanAddr::from(addr);
        assert_eq!("Hello, world!", human_addr_from_string.as_str());

        // &HumanAddr
        let human_addr_from_borrow = HumanAddr::from(&human_addr_from_string);
        assert_eq!(
            human_addr_from_borrow.as_str(),
            human_addr_from_string.as_str()
        );

        // &&HumanAddr
        let human_addr_from_borrow_2 = HumanAddr::from(&&human_addr_from_string);
        assert_eq!(
            human_addr_from_borrow_2.as_str(),
            human_addr_from_string.as_str()
        );
    }

    #[test]
    fn human_addr_implements_display() {
        let human_addr = HumanAddr::from("cos934gh9034hg04g0h134");
        let embedded = format!("Address: {}", human_addr);
        assert_eq!(embedded, "Address: cos934gh9034hg04g0h134");
        assert_eq!(human_addr.to_string(), "cos934gh9034hg04g0h134");
    }

    #[test]
    fn human_addr_len() {
        let addr = "Hello, world!";
        let human_addr = HumanAddr::from(addr);
        assert_eq!(addr.len(), human_addr.len());
    }

    #[test]
    fn human_addr_is_empty() {
        let human_addr = HumanAddr::from("Hello, world!");
        assert_eq!(false, human_addr.is_empty());
        let empty_human_addr = HumanAddr::from("");
        assert_eq!(true, empty_human_addr.is_empty());
    }

    // Test CanonicalAddr as_slice() for each CanonicalAddr::from input type
    #[test]
    fn canonical_addr_from_slice() {
        // slice
        let bytes: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let canonical_addr_slice = CanonicalAddr::from(bytes);
        assert_eq!(canonical_addr_slice.as_slice(), &[0u8, 187, 61, 11, 250, 0]);

        // Vector
        let bytes: Vec<u8> = vec![0u8, 187, 61, 11, 250, 0];
        let canonical_addr_vec = CanonicalAddr::from(bytes);
        assert_eq!(canonical_addr_vec.as_slice(), &[0u8, 187, 61, 11, 250, 0]);
    }

    #[test]
    fn canonical_addr_len() {
        let bytes: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let canonical_addr = CanonicalAddr::from(bytes);
        assert_eq!(canonical_addr.len(), bytes.len());
    }

    #[test]
    fn canonical_addr_is_empty() {
        let bytes: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let canonical_addr = CanonicalAddr::from(bytes);
        assert_eq!(false, canonical_addr.is_empty());
        let empty_canonical_addr = CanonicalAddr::from(vec![]);
        assert_eq!(true, empty_canonical_addr.is_empty());
    }
}
