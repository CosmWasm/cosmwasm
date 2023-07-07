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
        println!("Proto length: {}", encoded.len());
        let multi = MultiEncoding::decode(&*encoded).unwrap();

        assert_eq!(orig.name, multi.name);
        assert_eq!(orig.age, multi.age);

        let json = crate::to_vec(&multi).unwrap();
        println!("JSON length: {}", json.len());
        let serde: OnlySerde = crate::from_slice(&json).unwrap();

        assert_eq!(serde.name, multi.name);
        assert_eq!(serde.age, multi.age);
    }
}

#[cfg(test)]
mod newtype_tests {
    // cargo expand --tests --lib prost_demo::newtype_tests

    use cosmwasm_schema::{cw_prost_serde, cw_prost_serde_newtype};
    use prost::Message;

    #[cw_prost_serde]
    pub struct Name {
        // No way to flatten this
        #[prost(message, required, tag = "1")]
        pub name: Addr,
        #[prost(uint64, tag = "2")]
        pub age: u64,
    }

    // This wraps a struct, top level. As the wrapped object is a message / struct this is truly transparent
    #[cw_prost_serde_newtype]
    pub struct TransparentWrapper(pub Name);

    // This wraps a primitive and is embedded in a single field in a struct
    // Output is equivalent to:
    // #[cw_prost]
    // pub struct Addr {
    //     #[prost(string, tag = "1")]
    //     str: String,
    // }

    #[cw_prost_serde_newtype]
    pub struct Addr(String);

    impl Addr {
        pub fn new(addr: &str) -> Self {
            Addr(addr.to_string())
        }
    }

    // check out https://protobuf-decoder.netlify.app with
    // 0a090a0757696c6c69616d10a50a
    // (the output with cargo test -- --nocapture)
    // Both versions above produce the same output
    // Even when manually doing the wrapper of address it embeds one more layer

    #[test]
    fn encode_transparent_wrapper() {
        let name = Name {
            name: Addr::new("William"),
            age: 1317,
        };
        let wrapper = TransparentWrapper(name);
        let encoded = wrapper.encode_to_vec();
        let decoded = TransparentWrapper::decode(&*encoded).unwrap();

        println!("encoded: {:?}", hex::encode(encoded));

        assert_eq!(wrapper, decoded);
    }
}

#[cfg(test)]
mod u128_tests {
    use cosmwasm_schema::cw_prost;
    use prost::Message;

    // cargo expand --tests --lib prost_demo::u128_tests

    // No u128 support in protobuf: https://github.com/protocolbuffers/protobuf/issues/10963
    // we do it manually

    #[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug, Default, Copy)]
    struct Uint128(u128);

    impl ::prost::Message for Uint128 {
        fn encode_raw<B: ::prost::bytes::BufMut>(&self, buf: &mut B) {
            Uint128Impl::from(*self).encode_raw(buf)
        }

        fn clear(&mut self) {
            self.0 = 0u128;
        }

        #[inline]
        fn encoded_len(&self) -> usize {
            Uint128Impl::from(*self).encoded_len()
        }

        fn merge_field<B: ::prost::bytes::Buf>(
            &mut self,
            tag: u32,
            wire_type: ::prost::encoding::WireType,
            buf: &mut B,
            ctx: ::prost::encoding::DecodeContext,
        ) -> ::core::result::Result<(), ::prost::DecodeError> {
            let mut encoder = Uint128Impl::from(*self);
            encoder.merge_field(tag, wire_type, buf, ctx)?;
            let current = Uint128::from(encoder);
            *self = current;
            Ok(())
        }
    }

    #[cw_prost]
    struct Uint128Impl {
        #[prost(uint64, tag = "1")]
        pub low: u64,
        #[prost(uint64, tag = "2")]
        pub high: u64,
    }

    impl From<Uint128> for Uint128Impl {
        fn from(u: Uint128) -> Self {
            Uint128Impl {
                low: u.0 as u64,
                high: (u.0 >> 64) as u64,
            }
        }
    }

    impl From<Uint128Impl> for Uint128 {
        fn from(u: Uint128Impl) -> Self {
            Uint128((u.high as u128) << 64 | u.low as u128)
        }
    }

    #[test]
    fn proper_conversion() {
        let small = Uint128(12345678);
        let proto = Uint128Impl::from(small);
        let back = Uint128::from(proto);
        assert_eq!(small, back);

        let large = Uint128(u128::MAX);
        let proto = Uint128Impl::from(large);
        let back = Uint128::from(proto);
        assert_eq!(large, back);
    }

    #[test]
    fn encode_decode_u128() {
        let mut number = Uint128(123456789012345678901234567890);
        let encoded = number.encode_to_vec();
        let decoded = Uint128::decode(&*encoded).unwrap();
        assert_eq!(number, decoded);

        number.clear();
        let encoded = number.encode_to_vec();
        let decoded = Uint128::decode(&*encoded).unwrap();
        assert_eq!(Uint128(0), decoded);
    }

    #[test]
    fn encoding_size() {
        let number = Uint128(54_300_000);
        let proto_len = number.encoded_len();
        let json = crate::to_vec(&number).unwrap();
        let json_len = json.len();

        println!("proto: {}, json: {}", proto_len, json_len);
        assert!(proto_len < json_len);
    }
}
