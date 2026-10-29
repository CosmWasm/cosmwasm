#![allow(dead_code)]

use cosmwasm_schema::{
    cw_schema::{schema_of, EnumValue, Node, NodeType, Schema, Schemaifier, StructProperty},
    cw_serde,
};
use std::borrow::Cow;
use std::collections::BTreeMap;

#[cw_serde]
struct BridgeNested {
    retry_count: u32,
}

#[cw_serde]
#[serde(rename_all = "kebab-case")]
struct BridgeConfig {
    #[serde(rename = "wire-key")]
    api_key: String,
    #[serde(flatten)]
    nested: BridgeNested,
    #[serde(skip)]
    scratch: String,
}

#[cw_serde]
#[serde(tag = "kind", rename_all_fields = "camelCase")]
enum BridgeMsg {
    UpdateConfig {
        admin_addr: String,
    },
    #[serde(rename = "manual-stop")]
    StopJob {
        reason_code: u16,
    },
    #[serde(skip)]
    LocalOnly {},
}

#[cw_serde]
#[serde(rename_all(serialize = "PascalCase", deserialize = "snake_case"))]
struct BridgeDirectionalConfig {
    account_id: String,
    #[serde(rename(serialize = "wire-id", deserialize = "legacy_id"))]
    id: String,
}

#[cw_serde]
#[serde(rename_all_fields(serialize = "PascalCase", deserialize = "snake_case"))]
enum BridgeDirectionalMsg {
    UpdateConfig { admin_addr: String },
}

#[cw_serde]
#[serde(rename_all(deserialize = "snake_case", serialize = "PascalCase"))]
struct BridgeReverseDirectionalConfig {
    account_id: String,
    #[serde(rename(deserialize = "legacy_id", serialize = "wire-id"))]
    id: String,
}

#[cw_serde]
#[serde(rename_all_fields(deserialize = "snake_case", serialize = "kebab-case"))]
enum BridgeReverseDirectionalMsg {
    UpdateConfig { admin_addr: String },
}

#[cw_serde]
#[serde(rename_all = "kebab-case")]
enum BridgeCustomCaseMsg {
    StartJob,
}

#[cw_serde]
#[serde(rename_all(serialize = "PascalCase", deserialize = "kebab-case"))]
enum BridgeDirectionalCustomCaseMsg {
    StartJob,
}

#[cw_serde]
#[serde(rename_all(deserialize = "kebab-case"))]
enum BridgeDeserializeOnlyCaseMsg {
    StartJob,
}

#[cw_serde]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum BridgeTaggedCustomOrderMsg {
    StartJob {},
}

#[cw_serde]
#[serde(tag = "kind")]
#[serde(rename_all = "kebab-case")]
enum BridgeMultipleSerdeAttributesMsg {
    StartJob {},
}

#[cw_serde]
#[serde(untagged, rename_all = "kebab-case")]
enum BridgeUntaggedCustomOrderMsg {
    StartJob { account_id: String },
}

#[cw_serde]
#[serde(rename_all = "kebab-case")]
struct BridgeRawConfig {
    r#type: String,
    raw_value: u64,
}

fn bridge_default_retries() -> u32 {
    7
}

#[cw_serde]
struct BridgeDefaultConfig {
    #[serde(default = "bridge_default_retries")]
    retries: u32,
}

struct BridgeUnsupportedPayload;

#[cw_serde]
struct BridgeGenericSkipped<T> {
    visible: String,
    #[serde(skip)]
    hidden: T,
}

#[cw_serde]
enum BridgeGenericSkippedMsg<T> {
    Visible,
    #[serde(skip)]
    Hidden(T),
}

#[cw_serde]
enum BridgeRetainedSkippedFieldMsg<T> {
    Visible {
        visible: String,
        #[serde(skip)]
        hidden: Option<T>,
    },
}

fn root_node<T: Schemaifier>() -> Node {
    match schema_of::<T>() {
        Schema::V1(mut schema) => schema.definitions.remove(schema.root),
        _ => unreachable!("cw-schema currently emits v1 schemas"),
    }
}

fn struct_properties<T: Schemaifier>() -> BTreeMap<Cow<'static, str>, StructProperty> {
    match root_node::<T>().value {
        NodeType::Struct(cosmwasm_schema::cw_schema::StructType::Named { properties }) => {
            properties
        }
        other => panic!("expected named struct schema, got {other:?}"),
    }
}

fn enum_parts<T: Schemaifier>() -> (
    Option<Cow<'static, str>>,
    BTreeMap<Cow<'static, str>, cosmwasm_schema::cw_schema::EnumCase>,
) {
    match root_node::<T>().value {
        NodeType::Enum {
            discriminator,
            cases,
        } => (discriminator, cases),
        other => panic!("expected enum schema, got {other:?}"),
    }
}

fn named_case_properties<'a>(
    cases: &'a BTreeMap<Cow<'static, str>, cosmwasm_schema::cw_schema::EnumCase>,
    name: &str,
) -> &'a BTreeMap<Cow<'static, str>, StructProperty> {
    match &cases.get(name).unwrap().value {
        EnumValue::Named { properties } => properties,
        other => panic!("expected struct enum case, got {other:?}"),
    }
}

#[test]
fn cw_serde_struct_exposes_wire_field_names_00c7e4() {
    let properties = struct_properties::<BridgeConfig>();

    assert!(properties.contains_key("wire-key"));
    assert!(properties.contains_key("nested"));
    assert!(!properties.contains_key("api_key"));
    assert!(!properties.contains_key("scratch"));
}

#[test]
fn cw_serde_struct_preserves_flatten_marker_b6de31() {
    let properties = struct_properties::<BridgeConfig>();
    let nested = properties.get("nested").expect("nested property");

    assert!(nested.flattening);
    assert!(
        !properties
            .get("wire-key")
            .expect("wire-key property")
            .flattening
    );
}

#[test]
fn cw_serde_enum_exposes_tag_and_case_rules_f92a14() {
    let (discriminator, cases) = enum_parts::<BridgeMsg>();

    assert_eq!(discriminator.as_deref(), Some("kind"));
    assert!(cases.contains_key("update_config"));
    assert!(cases.contains_key("manual-stop"));
    assert!(!cases.contains_key("local_only"));
}

#[test]
fn cw_serde_enum_renames_struct_variant_fields_672bd9() {
    let (_, cases) = enum_parts::<BridgeMsg>();
    let update = named_case_properties(&cases, "update_config");
    let stop = named_case_properties(&cases, "manual-stop");

    assert!(update.contains_key("adminAddr"));
    assert!(!update.contains_key("admin_addr"));
    assert!(stop.contains_key("reasonCode"));
}

#[test]
fn cw_serde_directional_forms_use_serialize_names_3f5e91() {
    let properties = struct_properties::<BridgeDirectionalConfig>();
    assert!(properties.contains_key("AccountId"));
    assert!(properties.contains_key("wire-id"));
    assert!(!properties.contains_key("account_id"));
    assert!(!properties.contains_key("legacy_id"));

    let (_, cases) = enum_parts::<BridgeDirectionalMsg>();
    let update = named_case_properties(&cases, "update_config");
    assert!(update.contains_key("AdminAddr"));
    assert!(!update.contains_key("admin_addr"));
}

#[test]
fn cw_serde_reverse_order_directional_forms_use_serialize_names_f40d2b() {
    let properties = struct_properties::<BridgeReverseDirectionalConfig>();
    assert!(properties.contains_key("AccountId"));
    assert!(properties.contains_key("wire-id"));
    assert!(!properties.contains_key("account_id"));
    assert!(!properties.contains_key("legacy_id"));

    let (_, cases) = enum_parts::<BridgeReverseDirectionalMsg>();
    let update = named_case_properties(&cases, "update_config");
    assert!(update.contains_key("admin-addr"));
    assert!(!update.contains_key("admin_addr"));
}

#[test]
fn cw_serde_enum_explicit_rename_all_overrides_default_f532a8() {
    let (_, cases) = enum_parts::<BridgeCustomCaseMsg>();

    assert!(cases.contains_key("start-job"));
    assert!(!cases.contains_key("start_job"));
    assert_eq!(
        serde_json::to_string(&BridgeCustomCaseMsg::StartJob).unwrap(),
        r#""start-job""#
    );

    let (_, cases) = enum_parts::<BridgeDirectionalCustomCaseMsg>();
    assert!(cases.contains_key("StartJob"));
    assert!(!cases.contains_key("start_job"));
    assert_eq!(
        serde_json::to_string(&BridgeDirectionalCustomCaseMsg::StartJob).unwrap(),
        r#""StartJob""#
    );

    let (_, cases) = enum_parts::<BridgeDeserializeOnlyCaseMsg>();
    assert!(cases.contains_key("start_job"));
    assert!(!cases.contains_key("StartJob"));
    assert_eq!(
        serde_json::to_string(&BridgeDeserializeOnlyCaseMsg::StartJob).unwrap(),
        r#""start_job""#
    );
    assert_eq!(
        serde_json::from_str::<BridgeDeserializeOnlyCaseMsg>(r#""start-job""#).unwrap(),
        BridgeDeserializeOnlyCaseMsg::StartJob
    );
}

#[test]
fn cw_serde_explicit_rename_all_after_tag_or_untagged_is_detected_2d16b9() {
    let (discriminator, cases) = enum_parts::<BridgeTaggedCustomOrderMsg>();
    assert_eq!(discriminator.as_deref(), Some("kind"));
    assert!(cases.contains_key("start-job"));
    assert!(!cases.contains_key("start_job"));

    let (discriminator, cases) = enum_parts::<BridgeUntaggedCustomOrderMsg>();
    assert_eq!(discriminator, None);
    assert!(cases.contains_key("start-job"));
    assert!(!cases.contains_key("start_job"));
}

#[test]
fn cw_serde_composes_rename_all_from_later_serde_attribute_7a4e12() {
    let (discriminator, cases) = enum_parts::<BridgeMultipleSerdeAttributesMsg>();

    assert_eq!(discriminator.as_deref(), Some("kind"));
    assert!(cases.contains_key("start-job"));
    assert!(!cases.contains_key("start_job"));
    assert_eq!(
        serde_json::to_string(&BridgeMultipleSerdeAttributesMsg::StartJob {}).unwrap(),
        r#"{"kind":"start-job"}"#
    );
}

#[test]
fn cw_serde_default_and_retained_variant_skips_match_direct_893a0e() {
    assert!(
        struct_properties::<BridgeDefaultConfig>()
            .get("retries")
            .unwrap()
            .defaulting
    );

    let (_, cases) = enum_parts::<BridgeRetainedSkippedFieldMsg<BridgeUnsupportedPayload>>();
    let visible = named_case_properties(&cases, "visible");

    assert_eq!(visible.len(), 1);
    assert!(visible.contains_key("visible"));
}

#[test]
fn cw_serde_raw_identifiers_use_serde_names_cb27e4() {
    let properties = struct_properties::<BridgeRawConfig>();

    assert!(properties.contains_key("type"));
    assert!(properties.contains_key("raw-value"));
    assert!(!properties.contains_key("r#type"));
}

#[test]
fn cw_serde_skipped_generic_members_do_not_require_schemaifier_bounds_c8f195() {
    let properties = struct_properties::<BridgeGenericSkipped<BridgeUnsupportedPayload>>();
    assert_eq!(properties.len(), 1);
    assert!(properties.contains_key("visible"));

    let (_, cases) = enum_parts::<BridgeGenericSkippedMsg<BridgeUnsupportedPayload>>();
    assert_eq!(cases.len(), 1);
    assert!(cases.contains_key("visible"));
}
