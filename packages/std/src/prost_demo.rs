#[cfg(test)]
mod tests {
    use cosmwasm_schema::{cw_prost, cw_prost_serde, cw_serde};
    use prost::Message;

    // Note: it would be interesting to make this more transparent, using something like
    // https://docs.rs/autoproto/latest/autoproto/
    // However, that is 1000+ lines of proc macros with no tests and no commits since 2021.
    // Anyone want to fork that and maintain it?

    #[cw_prost]
    pub struct OnlyProto {
        #[prost(string, tag = "1")]
        pub name: String,
        #[prost(uint64, tag = "2")]
        pub age: u64,
    }

    #[cw_serde]
    pub struct OnlySerde {
        pub name: String,
        pub age: u64,
    }

    #[cw_prost_serde]
    pub struct MultiEncoding {
        #[prost(string, tag = "1")]
        pub name: String,
        #[prost(uint64, tag = "2")]
        pub age: u64,
    }

    #[test]
    fn encode_equivalence() {
        let orig = OnlyProto {
            name: "Billy".to_string(),
            age: 42,
        };
        let encoded = orig.encode_to_vec();
        let multi = MultiEncoding::decode(&*encoded).unwrap();

        assert_eq!(orig.name, multi.name);
        assert_eq!(orig.age, multi.age);

        let json = crate::to_vec(&multi).unwrap();
        let serde: OnlySerde = crate::from_slice(&json).unwrap();

        assert_eq!(serde.name, multi.name);
        assert_eq!(serde.age, multi.age);
    }
}
