#[cfg(test)]
mod basic_tests {
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

#[cfg(test)]
mod cosmwasm_tests {
    // cargo expand --tests --lib prost_demo::cosmwasm_tests

    use cosmwasm_schema::cw_prost;
    use prost::Message;

    #[cw_prost]
    pub struct Name {
        #[prost(string, tag = "1")]
        pub name: String,
        #[prost(uint64, tag = "2")]
        pub age: u64,
    }

    #[derive(Clone, PartialEq, Debug, Default)]
    pub struct TransparentWrapper(pub Name);

    // TODO: this needs to be another proc macro, like cw_wrap_proto (along with cw_wrap_proto_serde)
    impl ::prost::Message for TransparentWrapper {
        fn encode_raw<B: ::prost::bytes::BufMut>(&self, buf: &mut B) {
            self.0.encode_raw(buf)
        }

        fn clear(&mut self) {
            self.0.clear()
        }

        #[inline]
        fn encoded_len(&self) -> usize {
            self.0.encoded_len()
        }

        fn merge_field<B: ::prost::bytes::Buf>(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::WireType,
            buf: &mut B,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            self.0.merge_field(tag, wire_type, buf, ctx)
        }
    }

    #[test]
    fn encode_transparent_wrapper() {
        let name = Name {
            name: "William".to_string(),
            age: 1317,
        };
        let wrapper = TransparentWrapper(name.clone());
        let encoded = wrapper.encode_to_vec();
        let decoded = TransparentWrapper::decode(&*encoded).unwrap();

        assert_eq!(wrapper.0, decoded.0);
    }
}
