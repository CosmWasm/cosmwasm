#![allow(dead_code, non_snake_case, non_camel_case_types)]

use cosmwasm_schema::{
    cw_schema::{self, schema_of, EnumValue, NodeType, Schema, Schemaifier, StructType},
    cw_serde, QueryResponses,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;

struct Unsupported;

fn root_type<T: Schemaifier>() -> NodeType {
    match schema_of::<T>() {
        Schema::V1(schema) => schema.definitions[schema.root].value.clone(),
        _ => unreachable!(),
    }
}

fn assert_struct_names<T: Schemaifier + Serialize>(value: &T) {
    let NodeType::Struct(StructType::Named { properties }) = root_type::<T>() else {
        panic!("expected named struct");
    };
    let Value::Object(serialized) = serde_json::to_value(value).unwrap() else {
        panic!("expected serialized object");
    };
    assert_eq!(
        properties
            .keys()
            .map(|key| key.as_ref())
            .collect::<BTreeSet<_>>(),
        serialized
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
    );
}

fn assert_enum_names<T: Schemaifier + Serialize>(values: &[T]) {
    let NodeType::Enum { cases, .. } = root_type::<T>() else {
        panic!("expected enum");
    };
    let mut observed = BTreeSet::new();
    for value in values {
        let (name, fields) = match serde_json::to_value(value).unwrap() {
            Value::String(name) => (name, None),
            Value::Object(object) => {
                let (name, fields) = object.into_iter().next().unwrap();
                (name, Some(fields))
            }
            other => panic!("expected externally tagged enum, got {other:?}"),
        };
        let case = cases
            .get(name.as_str())
            .expect("serialized variant is in schema");
        if let Some(Value::Object(fields)) = fields {
            let EnumValue::Named { properties } = &case.value else {
                panic!("expected named variant fields");
            };
            assert_eq!(
                properties
                    .keys()
                    .map(|key| key.as_ref())
                    .collect::<BTreeSet<_>>(),
                fields.keys().map(String::as_str).collect::<BTreeSet<_>>()
            );
        }
        observed.insert(name);
    }
    assert_eq!(observed, cases.keys().map(ToString::to_string).collect());
}

macro_rules! naming_case {
    ($test:ident, $rule:literal) => {
        #[test]
        fn $test() {
            #[derive(Schemaifier, Serialize)]
            #[serde(rename_all = $rule)]
            struct DirectFields {
                r#type: bool,
                http2_server: bool,
                HTTPReady2: bool,
                _leading_value: bool,
            }

            #[cw_serde]
            #[serde(rename_all = $rule)]
            struct BridgeFields {
                r#type: bool,
                http2_server: bool,
                HTTPReady2: bool,
                _leading_value: bool,
            }

            #[derive(Schemaifier, Serialize)]
            #[serde(rename_all = $rule, rename_all_fields = $rule)]
            enum DirectCases {
                XMLParser2 { r#type: bool, field2_id: bool },
                Version42HTTP,
                r#async,
                V2Token,
            }

            #[cw_serde]
            #[serde(rename_all = $rule, rename_all_fields = $rule)]
            enum BridgeCases {
                XMLParser2 { r#type: bool, field2_id: bool },
                Version42HTTP,
                r#async,
                V2Token,
            }

            assert_struct_names(&DirectFields {
                r#type: true,
                http2_server: true,
                HTTPReady2: true,
                _leading_value: true,
            });
            assert_struct_names(&BridgeFields {
                r#type: true,
                http2_server: true,
                HTTPReady2: true,
                _leading_value: true,
            });
            assert_enum_names(&[
                DirectCases::XMLParser2 {
                    r#type: true,
                    field2_id: true,
                },
                DirectCases::Version42HTTP,
                DirectCases::r#async,
                DirectCases::V2Token,
            ]);
            assert_enum_names(&[
                BridgeCases::XMLParser2 {
                    r#type: true,
                    field2_id: true,
                },
                BridgeCases::Version42HTTP,
                BridgeCases::r#async,
                BridgeCases::V2Token,
            ]);
        }
    };
}

naming_case!(lowercase_matches_serde_for_direct_and_bridge, "lowercase");
naming_case!(uppercase_matches_serde_for_direct_and_bridge, "UPPERCASE");
naming_case!(
    pascal_case_matches_serde_for_direct_and_bridge,
    "PascalCase"
);
naming_case!(camel_case_matches_serde_for_direct_and_bridge, "camelCase");
naming_case!(snake_case_matches_serde_for_direct_and_bridge, "snake_case");
naming_case!(kebab_case_matches_serde_for_direct_and_bridge, "kebab-case");
naming_case!(
    screaming_snake_matches_serde_for_direct_and_bridge,
    "SCREAMING_SNAKE_CASE"
);
naming_case!(
    screaming_kebab_matches_serde_for_direct_and_bridge,
    "SCREAMING-KEBAB-CASE"
);

#[test]
fn cw_serde_tuple_skips_match_direct_schema() {
    #[derive(Schemaifier)]
    struct Direct<T>(#[serde(skip)] Option<T>, String);
    #[cw_serde]
    struct Bridge<T>(#[serde(skip)] Option<T>, String);
    #[cw_serde]
    enum BridgeEnum<T> {
        Present(#[serde(skip)] Option<T>, String),
    }

    for shape in [
        root_type::<Direct<Unsupported>>(),
        root_type::<Bridge<Unsupported>>(),
    ] {
        let NodeType::Struct(StructType::Tuple { items }) = shape else {
            panic!("expected tuple schema");
        };
        assert_eq!(items.len(), 1);
    }
    let NodeType::Enum { cases, .. } = root_type::<BridgeEnum<Unsupported>>() else {
        panic!("expected enum schema");
    };
    let EnumValue::Tuple { items } = &cases["present"].value else {
        panic!("expected tuple variant");
    };
    assert_eq!(items.len(), 1);
}

#[test]
fn cw_serde_qualified_types_do_not_bound_namesake_generic() {
    #[cw_serde]
    struct Bridge<String> {
        #[serde(rename = "wire-value")]
        value: std::string::String,
        #[serde(skip)]
        hidden: Option<String>,
    }
    #[cw_serde]
    enum BridgeEnum<String> {
        Visible(std::string::String),
        #[serde(skip)]
        Hidden(Option<String>),
    }
    let NodeType::Struct(StructType::Named { properties }) = root_type::<Bridge<Unsupported>>()
    else {
        panic!("expected named struct");
    };
    assert_eq!(properties.len(), 1);
    assert!(properties.contains_key("wire-value"));
    let NodeType::Enum { cases, .. } = root_type::<BridgeEnum<Unsupported>>() else {
        panic!("expected enum");
    };
    assert_eq!(cases.len(), 1);
    assert!(cases.contains_key("visible"));
}

#[test]
fn cw_serde_block_override_exempts_skipped_generic() {
    #[cw_serde]
    #[schemaifier(type = { cw_schema::NodeType::Unit })]
    struct Bridge<T> {
        #[serde(skip)]
        hidden: Option<T>,
    }
    assert_eq!(root_type::<Bridge<Unsupported>>(), NodeType::Unit);
}

#[test]
fn direct_attributes_compose_across_groups_and_order() {
    #[derive(Schemaifier, Serialize)]
    #[serde(rename_all_fields = "camelCase")]
    #[serde(tag = "wire-kind")]
    #[serde(rename_all = "kebab-case")]
    enum Split {
        #[serde(rename(serialize = "send-now"))]
        #[serde(rename(deserialize = "old-send"))]
        SendNow {
            #[serde(default)]
            #[serde(rename(serialize = "wire-count"))]
            #[serde(rename(deserialize = "old-count"))]
            retry_count: u32,
            account_id: String,
        },
        #[serde(rename = "private")]
        #[serde(skip)]
        LocalOnly,
    }

    #[derive(Schemaifier, Serialize)]
    #[serde(
        rename_all = "kebab-case",
        tag = "wire-kind",
        rename_all_fields = "camelCase"
    )]
    enum Grouped {
        #[serde(rename(deserialize = "old-send", serialize = "send-now"))]
        SendNow {
            #[serde(rename(deserialize = "old-count", serialize = "wire-count"), default)]
            retry_count: u32,
            account_id: String,
        },
        #[serde(skip, rename = "private")]
        LocalOnly,
    }

    for shape in [root_type::<Split>(), root_type::<Grouped>()] {
        let NodeType::Enum {
            discriminator,
            cases,
        } = shape
        else {
            panic!("expected enum");
        };
        assert_eq!(discriminator.as_deref(), Some("wire-kind"));
        assert_eq!(cases.len(), 1);
        let EnumValue::Named { properties } = &cases["send-now"].value else {
            panic!("expected named variant");
        };
        assert_eq!(properties.len(), 2);
        assert!(properties["wire-count"].defaulting);
        assert!(!properties["accountId"].defaulting);
    }
    let expected = json!({"wire-kind": "send-now", "wire-count": 3, "accountId": "a"});
    assert_eq!(
        serde_json::to_value(Split::SendNow {
            retry_count: 3,
            account_id: "a".into()
        })
        .unwrap(),
        expected
    );
    assert_eq!(
        serde_json::to_value(Grouped::SendNow {
            retry_count: 3,
            account_id: "a".into()
        })
        .unwrap(),
        expected
    );
}

#[test]
fn default_query_names_match_schema_serialization_and_deserialization() {
    #[cw_serde]
    #[derive(QueryResponses)]
    enum Query {
        #[returns(bool)]
        AccountInfo {
            #[serde(rename = "account-key")]
            account_key: String,
        },
        #[returns(u64)]
        NodeStats {},
    }

    let NodeType::Enum { cases, .. } = root_type::<Query>() else {
        panic!("expected query enum schema");
    };
    let expected = BTreeSet::from(["account_info", "node_stats"]);
    assert_eq!(
        cases
            .keys()
            .map(|key| key.as_ref())
            .collect::<BTreeSet<_>>(),
        expected
    );
    assert_eq!(
        Query::response_schemas()
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        expected
    );
    assert_eq!(
        Query::response_schemas_cw()
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        expected
    );
    let EnumValue::Named { properties } = &cases["account_info"].value else {
        panic!("expected named query fields");
    };
    assert!(properties.contains_key("account-key"));
    let wire = json!({"account_info": {"account-key": "alice"}});
    let query: Query = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(
        query,
        Query::AccountInfo {
            account_key: "alice".into()
        }
    );
    assert_eq!(serde_json::to_value(query).unwrap(), wire);
    assert!(
        serde_json::from_value::<Query>(json!({"AccountInfo": {"account-key": "alice"}})).is_err()
    );
}

#[test]
fn named_struct_attributes_compose_for_direct_and_cw_serde() {
    #[cw_serde]
    #[derive(Default)]
    struct Tail {
        tail_value: bool,
    }

    macro_rules! split_struct {
        ($name:ident, $($derive:tt)*) => {
            $($derive)*
            #[serde(rename_all(serialize = "kebab-case"))]
            #[serde(rename_all(deserialize = "camelCase"))]
            struct $name<T> {
                plain_field: bool,
                #[serde(rename(serialize = "wire-count"))]
                #[serde(default)]
                #[serde(rename(deserialize = "old-count"))]
                retry_count: u32,
                #[serde(flatten)]
                #[serde(rename = "more-options")]
                #[serde(default)]
                more: Tail,
                #[serde(rename = "private-state")]
                #[serde(skip)]
                hidden: Option<T>,
            }
        };
    }
    macro_rules! grouped_struct {
        ($name:ident, $($derive:tt)*) => {
            $($derive)*
            #[serde(rename_all(deserialize = "camelCase", serialize = "kebab-case"))]
            struct $name<T> {
                plain_field: bool,
                #[serde(default, rename(deserialize = "old-count", serialize = "wire-count"))]
                retry_count: u32,
                #[serde(default, rename = "more-options", flatten)]
                more: Tail,
                #[serde(skip, rename = "private-state")]
                hidden: Option<T>,
            }
        };
    }
    split_struct!(DirectSplit, #[derive(Schemaifier, Serialize)]);
    grouped_struct!(DirectGrouped, #[derive(Schemaifier, Serialize)]);
    split_struct!(BridgeSplit, #[cw_serde]);
    grouped_struct!(BridgeGrouped, #[cw_serde]);

    macro_rules! check {
        ($ty:ident) => {
            let NodeType::Struct(StructType::Named { properties }) = root_type::<$ty<Unsupported>>() else {
                panic!("expected named struct");
            };
            assert_eq!(properties.keys().map(|key| key.as_ref()).collect::<BTreeSet<_>>(),
                BTreeSet::from(["plain-field", "wire-count", "more-options"]));
            assert!(!properties["plain-field"].defaulting);
            assert!(!properties["plain-field"].flattening);
            assert!(properties["wire-count"].defaulting);
            assert!(!properties["wire-count"].flattening);
            assert!(properties["more-options"].defaulting);
            assert!(properties["more-options"].flattening);
            assert_eq!(serde_json::to_value(&properties["more-options"]).unwrap()["flattening"], true);
            let value = $ty::<Unsupported> {
                plain_field: true,
                retry_count: 3,
                more: Tail { tail_value: true },
                hidden: None,
            };
            assert_eq!(serde_json::to_value(value).unwrap(),
                json!({"plain-field": true, "wire-count": 3, "tail_value": true}));
        };
    }
    check!(DirectSplit);
    check!(DirectGrouped);
    check!(BridgeSplit);
    check!(BridgeGrouped);
}

#[test]
fn value_default_then_rename_composes_for_direct_and_cw_serde() {
    fn default_retries() -> u32 {
        7
    }
    macro_rules! declare {
        ($name:ident, $($derive:tt)*) => {
            $($derive)*
            struct $name {
                #[serde(default = "default_retries", rename(serialize = "wire-retries", deserialize = "legacy-retries"))]
                retry_count: u32,
                enabled: bool,
            }
        };
    }
    declare!(Direct, #[derive(Schemaifier, Serialize, serde::Deserialize)]);
    declare!(Bridge, #[cw_serde]);

    macro_rules! check {
        ($ty:ident) => {
            let NodeType::Struct(StructType::Named { properties }) = root_type::<$ty>() else {
                panic!("expected named struct");
            };
            assert_eq!(properties.keys().map(|key| key.as_ref()).collect::<BTreeSet<_>>(),
                BTreeSet::from(["wire-retries", "enabled"]));
            assert!(properties["wire-retries"].defaulting);
            assert!(!properties["enabled"].defaulting);
            let value: $ty = serde_json::from_value(json!({"enabled": true})).unwrap();
            assert_eq!(value.retry_count, 7);
            assert_eq!(serde_json::to_value(value).unwrap(), json!({"wire-retries": 7, "enabled": true}));
        };
    }
    check!(Direct);
    check!(Bridge);
}

#[test]
fn value_default_then_flatten_composes_for_direct_and_cw_serde() {
    #[cw_serde]
    struct Tail {
        tail_value: bool,
    }
    fn default_tail() -> Tail {
        Tail { tail_value: false }
    }
    macro_rules! declare {
        ($name:ident, $($derive:tt)*) => {
            $($derive)*
            struct $name {
                enabled: bool,
                #[serde(default = "default_tail", flatten, rename = "options")]
                tail: Tail,
            }
        };
    }
    declare!(Direct, #[derive(Schemaifier, Serialize)]);
    declare!(Bridge, #[cw_serde]);

    macro_rules! check {
        ($ty:ident) => {
            let NodeType::Struct(StructType::Named { properties }) = root_type::<$ty>() else {
                panic!("expected named struct");
            };
            assert_eq!(properties.keys().map(|key| key.as_ref()).collect::<BTreeSet<_>>(),
                BTreeSet::from(["options", "enabled"]));
            assert!(properties["options"].defaulting);
            assert!(properties["options"].flattening);
            assert!(!properties["enabled"].flattening);
            assert_eq!(serde_json::to_value(&properties["options"]).unwrap()["flattening"], true);
            let value = $ty { enabled: true, tail: Tail { tail_value: false } };
            assert_eq!(serde_json::to_value(value).unwrap(), json!({"enabled": true, "tail_value": false}));
        };
    }
    check!(Direct);
    check!(Bridge);
}

#[test]
fn value_default_then_skip_composes_for_direct_and_cw_serde() {
    fn no_value<T>() -> Option<T> {
        None
    }
    macro_rules! declare {
        ($name:ident, $($derive:tt)*) => {
            $($derive)*
            struct $name<T> {
                #[serde(default = "no_value", skip)]
                hidden: Option<T>,
                enabled: bool,
            }
        };
    }
    declare!(Direct, #[derive(Schemaifier, Serialize)]);
    declare!(Bridge, #[cw_serde]);

    macro_rules! check {
        ($ty:ident) => {
            let NodeType::Struct(StructType::Named { properties }) = root_type::<$ty<Unsupported>>() else {
                panic!("expected named struct");
            };
            assert_eq!(properties.keys().map(|key| key.as_ref()).collect::<BTreeSet<_>>(),
                BTreeSet::from(["enabled"]));
            let value = $ty::<Unsupported> { hidden: None, enabled: true };
            assert_eq!(serde_json::to_value(value).unwrap(), json!({"enabled": true}));
        };
    }
    check!(Direct);
    check!(Bridge);
}
