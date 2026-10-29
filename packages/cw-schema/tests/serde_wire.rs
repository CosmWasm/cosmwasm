#![allow(dead_code)]

use cw_schema::{
    schema_of, EnumValue, Node, NodeType, Schema, Schemaifier, StructProperty, StructType,
};
use std::borrow::Cow;
use std::collections::BTreeMap;

#[derive(Schemaifier)]
struct NestedConfig {
    timeout_seconds: u64,
}

#[derive(Schemaifier)]
#[serde(rename_all = "kebab-case")]
struct WireConfig {
    plain_field: String,
    #[serde(rename = "wire-id")]
    identifier: String,
    #[serde(default)]
    retries: u32,
    #[serde(flatten)]
    nested: NestedConfig,
    #[serde(skip)]
    local_cache: Vec<u8>,
}

#[derive(Schemaifier)]
#[serde(
    tag = "action",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum WireAction {
    StartJob {
        account_id: String,
        #[serde(rename = "max-retries")]
        retry_count: u32,
    },
    #[serde(rename = "manual-stop")]
    StopJob { reason_code: u16 },
    #[serde(skip)]
    LocalOnly,
}

#[derive(Schemaifier)]
#[serde(untagged, rename_all = "SCREAMING-KEBAB-CASE")]
enum UntaggedChoice {
    Text(String),
    #[serde(rename = "bytes-choice")]
    Bytes(Vec<u8>),
}

#[derive(Schemaifier)]
#[serde(rename_all = "PascalCase")]
struct PascalMessage {
    account_id: String,
}

#[derive(Schemaifier)]
#[serde(rename_all = "lowercase")]
struct LowerMessage {
    account_id: String,
}

#[derive(Schemaifier)]
#[serde(rename_all = "UPPERCASE")]
struct UpperMessage {
    account_id: String,
}

#[derive(Schemaifier)]
#[serde(rename_all = "UPPERCASE")]
enum UpperChoice {
    FirstOption,
    SecondOption,
}

#[derive(Schemaifier)]
#[serde(
    rename_all(serialize = "kebab-case", deserialize = "snake_case"),
    rename_all_fields(serialize = "camelCase", deserialize = "snake_case")
)]
enum DirectionalAction {
    RunJob {
        account_id: String,
    },
    #[serde(rename(serialize = "manual-stop", deserialize = "legacy_stop"))]
    StopJob {
        reason_code: u16,
    },
}

#[derive(Schemaifier)]
enum DirectionalVariantField {
    Update {
        #[serde(rename(serialize = "wire-field", deserialize = "legacy_field"))]
        field: String,
    },
}

#[derive(Schemaifier)]
#[serde(rename_all(serialize = "kebab-case", deserialize = "snake_case"))]
struct DirectionalConfig {
    #[serde(rename(serialize = "wire-id", deserialize = "legacy_id"))]
    id: String,
    plain_field: bool,
}

#[derive(Schemaifier)]
#[serde(rename_all(deserialize = "snake_case", serialize = "kebab-case"))]
struct ReverseDirectionalConfig {
    #[serde(rename(deserialize = "legacy_id", serialize = "wire-id"))]
    id: String,
    plain_field: bool,
}

#[derive(Schemaifier)]
#[serde(
    rename_all(deserialize = "snake_case", serialize = "kebab-case"),
    rename_all_fields(deserialize = "snake_case", serialize = "PascalCase")
)]
enum ReverseDirectionalAction {
    RunJob {
        account_id: String,
    },
    #[serde(rename(deserialize = "legacy_stop", serialize = "manual-stop"))]
    StopJob {
        reason_code: u16,
    },
}

fn default_retries() -> u32 {
    3
}

struct UnsupportedPayload;

macro_rules! case_field_type {
    ($name:ident, $rule:literal) => {
        #[derive(Schemaifier)]
        #[serde(rename_all = $rule)]
        struct $name {
            http_server2: String,
        }
    };
}

macro_rules! case_variant_type {
    ($name:ident, $rule:literal) => {
        #[derive(Schemaifier)]
        #[serde(rename_all = $rule)]
        enum $name {
            HTTPServer2,
        }
    };
}

case_field_type!(AllCaseFieldLower, "lowercase");
case_field_type!(AllCaseFieldUpper, "UPPERCASE");
case_field_type!(AllCaseFieldPascal, "PascalCase");
case_field_type!(AllCaseFieldCamel, "camelCase");
case_field_type!(AllCaseFieldSnake, "snake_case");
case_field_type!(AllCaseFieldKebab, "kebab-case");
case_field_type!(AllCaseFieldScreamingSnake, "SCREAMING_SNAKE_CASE");
case_field_type!(AllCaseFieldScreamingKebab, "SCREAMING-KEBAB-CASE");

case_variant_type!(AllCaseVariantLower, "lowercase");
case_variant_type!(AllCaseVariantUpper, "UPPERCASE");
case_variant_type!(AllCaseVariantPascal, "PascalCase");
case_variant_type!(AllCaseVariantCamel, "camelCase");
case_variant_type!(AllCaseVariantSnake, "snake_case");
case_variant_type!(AllCaseVariantKebab, "kebab-case");
case_variant_type!(AllCaseVariantScreamingSnake, "SCREAMING_SNAKE_CASE");
case_variant_type!(AllCaseVariantScreamingKebab, "SCREAMING-KEBAB-CASE");

#[derive(Schemaifier)]
struct DefaultFunctionConfig {
    #[serde(default = "default_retries")]
    retries: u32,
}

#[derive(Schemaifier)]
#[serde(rename_all(deserialize = "kebab-case"))]
struct DeserializeOnlyConfig {
    plain_field: bool,
    #[serde(rename(deserialize = "legacy-id"))]
    directed_field: bool,
}

#[derive(Schemaifier)]
#[serde(rename_all(deserialize = "snake_case"))]
enum DeserializeOnlyVariants {
    RunJob,
    #[serde(rename(deserialize = "legacy-stop"))]
    StopJob,
}

#[derive(Schemaifier)]
#[serde(rename_all_fields(deserialize = "camelCase"))]
enum DeserializeOnlyFields {
    Account { account_id: String },
}

#[derive(Schemaifier)]
struct SkippedUnsupportedField {
    visible: String,
    #[serde(skip)]
    hidden: UnsupportedPayload,
}

#[derive(Schemaifier)]
struct GenericSkippedUnsupportedField<T> {
    visible: String,
    #[serde(skip)]
    hidden: T,
}

#[derive(Schemaifier)]
enum SkippedUnsupportedVariant {
    Visible,
    #[serde(skip)]
    Hidden(UnsupportedPayload),
}

#[derive(Schemaifier)]
enum GenericSkippedUnsupportedVariant<T> {
    Visible,
    #[serde(skip)]
    Hidden(T),
}

#[derive(Schemaifier)]
enum GenericSkippedUnsupportedVariantField<T> {
    Present {
        visible: String,
        #[serde(skip)]
        hidden: T,
    },
}

#[derive(Schemaifier)]
struct NestedGenericSkippedField<T> {
    visible: String,
    #[serde(skip)]
    hidden: Option<Vec<T>>,
}

#[derive(Schemaifier)]
enum NestedGenericSkippedVariant<T> {
    Visible,
    #[serde(skip)]
    Hidden(Option<Vec<T>>),
}

#[derive(Schemaifier)]
enum NestedGenericSkippedVariantField<T> {
    Present {
        visible: String,
        #[serde(skip)]
        hidden: Option<Vec<T>>,
    },
}

#[derive(Schemaifier)]
struct MixedGenericField<T, U> {
    visible: T,
    #[serde(skip)]
    hidden: Option<U>,
}

#[derive(Schemaifier)]
enum MixedGenericVariant<T, U> {
    Present(T),
    #[serde(skip)]
    Hidden(Option<U>),
}

#[derive(Schemaifier)]
enum MixedGenericVariantField<T, U> {
    Present {
        visible: T,
        #[serde(skip)]
        hidden: Option<U>,
    },
}

#[derive(Schemaifier)]
struct QualifiedPathCollision<String> {
    visible: std::string::String,
    #[serde(skip)]
    hidden: String,
}

#[derive(Schemaifier)]
enum QualifiedEnumPathCollision<String> {
    Visible(std::string::String),
    #[serde(skip)]
    Hidden(String),
}

#[derive(Schemaifier)]
struct TupleSkippedUnsupported(#[serde(skip)] UnsupportedPayload, String);

#[derive(Schemaifier)]
enum TupleVariantSkippedUnsupported {
    Present(#[serde(skip)] UnsupportedPayload, String),
}

#[derive(Schemaifier)]
#[serde(rename_all = "kebab-case")]
enum VariantOnlyRenameAll {
    AccountEvent { account_id: String },
}

#[derive(Schemaifier)]
#[serde(
    rename_all(serialize = "kebab-case"),
    rename_all_fields(serialize = "camelCase")
)]
enum SerializeOnlyDirectionalAction {
    RunJob {
        account_id: String,
    },
    #[serde(rename(serialize = "manual-stop"))]
    StopJob {
        reason_code: u16,
    },
}

#[derive(Schemaifier)]
#[schemaifier(type = cw_schema::NodeType::Unit)]
struct TypeOverrideSkippedGeneric<T> {
    #[serde(skip)]
    hidden: T,
}

#[derive(Schemaifier)]
#[serde(tag = "wire-tag")]
enum NonIdentifierTag {
    Start {},
}

#[derive(Schemaifier)]
#[serde(rename_all = "kebab-case")]
struct RawKeywordConfig {
    r#type: String,
    raw_value: u64,
}

#[allow(non_camel_case_types)]
#[derive(Schemaifier)]
#[serde(rename_all = "UPPERCASE")]
enum RawKeywordChoice {
    r#type,
    ValueKind,
}

#[test]
fn struct_fields_follow_serde_wire_names_2e22b4() {
    let properties = struct_properties::<WireConfig>();

    assert!(properties.contains_key("plain-field"));
    assert!(properties.contains_key("wire-id"));
    assert!(properties.contains_key("retries"));
    assert!(properties.contains_key("nested"));
    assert!(!properties.contains_key("plain_field"));
    assert!(!properties.contains_key("identifier"));
    assert!(!properties.contains_key("local_cache"));
    assert!(!properties.contains_key("local-cache"));

    assert_flattening(&properties, "nested", true);
    assert_flattening(&properties, "plain-field", false);

    let encoded = serde_json::to_value(properties.get("nested").unwrap()).unwrap();

    assert_eq!(
        encoded.get("flattening"),
        Some(&serde_json::Value::Bool(true))
    );
    assert!(encoded.get("flatten").is_none());
    assert!(encoded.get("flattened").is_none());

    assert!(properties.get("retries").unwrap().defaulting);
    assert!(!properties.get("wire-id").unwrap().defaulting);
}

#[test]
fn enum_tag_and_serde_case_rules_are_exposed_34f5d6() {
    let (discriminator, cases) = enum_parts::<WireAction>();

    assert_eq!(discriminator.as_deref(), Some("action"));
    assert!(cases.contains_key("start_job"));
    assert!(cases.contains_key("manual-stop"));
    assert!(!cases.contains_key("StartJob"));
}

#[test]
fn enum_rename_all_fields_only_renames_struct_variant_fields_ba4019() {
    let (_, cases) = enum_parts::<WireAction>();

    let start = named_case_properties(&cases, "start_job");
    assert!(start.contains_key("accountId"));
    assert!(start.contains_key("max-retries"));
    assert!(!start.contains_key("account_id"));

    let stop = named_case_properties(&cases, "manual-stop");
    assert!(stop.contains_key("reasonCode"));
}

#[test]
fn enum_skip_omits_variant_from_schema_4cd279() {
    let (_, cases) = enum_parts::<WireAction>();

    assert!(!cases.contains_key("local_only"));
    assert!(!cases.contains_key("LocalOnly"));
}

#[test]
fn untagged_enum_has_no_discriminator_but_keeps_case_names_cfe96a() {
    let (discriminator, cases) = enum_parts::<UntaggedChoice>();

    assert_eq!(discriminator, None);
    assert!(cases.contains_key("TEXT"));
    assert!(cases.contains_key("bytes-choice"));
}

#[test]
fn all_serde_case_rules_cover_fields_variants_and_acronyms_6ad927() {
    let field_names = [
        first_property_name::<AllCaseFieldLower>(),
        first_property_name::<AllCaseFieldUpper>(),
        first_property_name::<AllCaseFieldPascal>(),
        first_property_name::<AllCaseFieldCamel>(),
        first_property_name::<AllCaseFieldSnake>(),
        first_property_name::<AllCaseFieldKebab>(),
        first_property_name::<AllCaseFieldScreamingSnake>(),
        first_property_name::<AllCaseFieldScreamingKebab>(),
    ];
    assert_eq!(
        field_names,
        [
            "http_server2",
            "HTTP_SERVER2",
            "HttpServer2",
            "httpServer2",
            "http_server2",
            "http-server2",
            "HTTP_SERVER2",
            "HTTP-SERVER2",
        ]
    );

    let variant_names = [
        first_case_name::<AllCaseVariantLower>(),
        first_case_name::<AllCaseVariantUpper>(),
        first_case_name::<AllCaseVariantPascal>(),
        first_case_name::<AllCaseVariantCamel>(),
        first_case_name::<AllCaseVariantSnake>(),
        first_case_name::<AllCaseVariantKebab>(),
        first_case_name::<AllCaseVariantScreamingSnake>(),
        first_case_name::<AllCaseVariantScreamingKebab>(),
    ];
    assert_eq!(
        variant_names,
        [
            "httpserver2",
            "HTTPSERVER2",
            "HTTPServer2",
            "hTTPServer2",
            "h_t_t_p_server2",
            "h-t-t-p-server2",
            "H_T_T_P_SERVER2",
            "H-T-T-P-SERVER2",
        ]
    );
}

#[test]
fn directional_rename_forms_use_serialize_names_41d0ac() {
    let properties = struct_properties::<DirectionalConfig>();
    assert!(properties.contains_key("wire-id"));
    assert!(properties.contains_key("plain-field"));
    assert!(!properties.contains_key("legacy_id"));
    assert!(!properties.contains_key("plain_field"));

    let (_, cases) = enum_parts::<DirectionalAction>();
    assert!(cases.contains_key("run-job"));
    assert!(cases.contains_key("manual-stop"));
    assert!(!cases.contains_key("legacy_stop"));

    let run = named_case_properties(&cases, "run-job");
    let stop = named_case_properties(&cases, "manual-stop");

    assert!(run.contains_key("accountId"));
    assert!(stop.contains_key("reasonCode"));
    assert!(!run.contains_key("account_id"));

    let properties = struct_properties::<ReverseDirectionalConfig>();
    assert!(properties.contains_key("wire-id"));
    assert!(properties.contains_key("plain-field"));
    assert!(!properties.contains_key("legacy_id"));
    assert!(!properties.contains_key("plain_field"));

    let (_, cases) = enum_parts::<ReverseDirectionalAction>();
    assert!(cases.contains_key("run-job"));
    assert!(cases.contains_key("manual-stop"));
    assert!(!cases.contains_key("legacy_stop"));

    let run = named_case_properties(&cases, "run-job");
    let stop = named_case_properties(&cases, "manual-stop");

    assert!(run.contains_key("AccountId"));
    assert!(stop.contains_key("ReasonCode"));
    assert!(!run.contains_key("account_id"));

    let (_, cases) = enum_parts::<SerializeOnlyDirectionalAction>();
    assert!(cases.contains_key("run-job"));
    assert!(cases.contains_key("manual-stop"));

    let run = named_case_properties(&cases, "run-job");
    let stop = named_case_properties(&cases, "manual-stop");

    assert!(run.contains_key("accountId"));
    assert!(stop.contains_key("reasonCode"));
    assert!(!run.contains_key("account_id"));
}

#[test]
fn enum_struct_field_directional_rename_uses_serialize_name_83c91a() {
    let (_, cases) = enum_parts::<DirectionalVariantField>();
    let update = named_case_properties(&cases, "Update");

    assert!(update.contains_key("wire-field"));
    assert!(!update.contains_key("legacy_field"));
    assert!(!update.contains_key("field"));
}

#[test]
fn default_function_form_sets_defaulting_6b64ae() {
    assert!(
        struct_properties::<DefaultFunctionConfig>()
            .get("retries")
            .unwrap()
            .defaulting
    );
}

#[test]
fn deserialize_only_renames_do_not_change_serialize_schema_names_749d58() {
    let properties = struct_properties::<DeserializeOnlyConfig>();
    assert!(properties.contains_key("plain_field"));
    assert!(properties.contains_key("directed_field"));
    assert!(!properties.contains_key("plain-field"));
    assert!(!properties.contains_key("legacy-id"));

    let (_, cases) = enum_parts::<DeserializeOnlyVariants>();
    assert!(cases.contains_key("RunJob"));
    assert!(cases.contains_key("StopJob"));
    assert!(!cases.contains_key("run_job"));
    assert!(!cases.contains_key("legacy-stop"));

    let (_, cases) = enum_parts::<DeserializeOnlyFields>();
    let account = named_case_properties(&cases, "Account");
    assert!(account.contains_key("account_id"));
    assert!(!account.contains_key("accountId"));
}

#[test]
fn skipped_members_do_not_require_schemaifier_payloads_11b2cf() {
    let properties = struct_properties::<SkippedUnsupportedField>();
    assert_eq!(properties.len(), 1);
    assert!(properties.contains_key("visible"));

    let (_, cases) = enum_parts::<SkippedUnsupportedVariant>();
    assert_eq!(cases.len(), 1);
    assert!(cases.contains_key("Visible"));
}

#[test]
fn skipped_generic_members_do_not_require_schemaifier_bounds_2bf0c9() {
    let properties = struct_properties::<GenericSkippedUnsupportedField<UnsupportedPayload>>();
    assert_eq!(properties.len(), 1);
    assert!(properties.contains_key("visible"));

    let (_, cases) = enum_parts::<GenericSkippedUnsupportedVariant<UnsupportedPayload>>();
    assert_eq!(cases.len(), 1);
    assert!(cases.contains_key("Visible"));
}

#[test]
fn retained_enum_variant_skipped_generic_field_has_no_schemaifier_bound_62f973() {
    let (_, cases) = enum_parts::<GenericSkippedUnsupportedVariantField<UnsupportedPayload>>();
    let present = named_case_properties(&cases, "Present");

    assert_eq!(present.len(), 1);
    assert!(present.contains_key("visible"));
}

#[test]
fn skipped_nested_generic_payloads_and_retained_uses_have_correct_bounds_71c3e9() {
    let properties = struct_properties::<NestedGenericSkippedField<UnsupportedPayload>>();
    assert_eq!(properties.len(), 1);
    assert!(properties.contains_key("visible"));

    let (_, cases) = enum_parts::<NestedGenericSkippedVariant<UnsupportedPayload>>();
    assert_eq!(cases.len(), 1);
    assert!(cases.contains_key("Visible"));

    let (_, cases) = enum_parts::<NestedGenericSkippedVariantField<UnsupportedPayload>>();
    let present = named_case_properties(&cases, "Present");
    assert_eq!(present.len(), 1);
    assert!(present.contains_key("visible"));

    let properties = struct_properties::<MixedGenericField<String, UnsupportedPayload>>();
    assert_eq!(properties.len(), 1);
    assert!(properties.contains_key("visible"));

    let (_, cases) = enum_parts::<MixedGenericVariant<String, UnsupportedPayload>>();
    assert_eq!(cases.len(), 1);
    assert!(cases.contains_key("Present"));

    let (_, cases) = enum_parts::<MixedGenericVariantField<String, UnsupportedPayload>>();
    let present = named_case_properties(&cases, "Present");
    assert_eq!(present.len(), 1);
    assert!(present.contains_key("visible"));
}

#[test]
fn qualified_paths_do_not_match_generic_parameter_names_d49be7() {
    let properties = struct_properties::<QualifiedPathCollision<UnsupportedPayload>>();
    assert_eq!(properties.len(), 1);
    assert!(properties.contains_key("visible"));

    let (_, cases) = enum_parts::<QualifiedEnumPathCollision<UnsupportedPayload>>();
    assert_eq!(cases.len(), 1);
    assert!(cases.contains_key("Visible"));
}

#[test]
fn skipped_tuple_fields_do_not_require_schemaifier_payloads_97a6e2() {
    assert_eq!(tuple_item_count::<TupleSkippedUnsupported>(), 1);

    let (_, cases) = enum_parts::<TupleVariantSkippedUnsupported>();
    match &cases.get("Present").unwrap().value {
        EnumValue::Tuple { items } => assert_eq!(items.len(), 1),
        other => panic!("expected tuple enum case, got {other:?}"),
    }
}

#[test]
fn rename_all_without_rename_all_fields_keeps_struct_variant_fields_492b1d() {
    let (_, cases) = enum_parts::<VariantOnlyRenameAll>();
    let event = named_case_properties(&cases, "account-event");

    assert!(event.contains_key("account_id"));
    assert!(!event.contains_key("account-id"));
}

#[test]
fn schemaifier_type_override_does_not_bound_skipped_generic_payload_4fd832() {
    let node = root_node::<TypeOverrideSkippedGeneric<UnsupportedPayload>>();
    assert!(matches!(node.value, NodeType::Unit));
}

#[test]
fn enum_tag_preserves_non_identifier_string_exactly_1cff49() {
    let (discriminator, _) = enum_parts::<NonIdentifierTag>();
    assert_eq!(discriminator.as_deref(), Some("wire-tag"));
}

#[test]
fn raw_identifiers_use_serde_wire_names_0ab6d8() {
    let properties = struct_properties::<RawKeywordConfig>();
    assert!(properties.contains_key("type"));
    assert!(properties.contains_key("raw-value"));
    assert!(!properties.contains_key("r#type"));

    let (_, cases) = enum_parts::<RawKeywordChoice>();
    assert!(cases.contains_key("TYPE"));
    assert!(cases.contains_key("VALUEKIND"));
    assert!(!cases.contains_key("R#TYPE"));
}

fn first_property_name<T: Schemaifier>() -> String {
    struct_properties::<T>().keys().next().unwrap().to_string()
}

fn first_case_name<T: Schemaifier>() -> String {
    enum_parts::<T>().1.keys().next().unwrap().to_string()
}

fn root_node<T: Schemaifier>() -> Node {
    match schema_of::<T>() {
        Schema::V1(mut schema) => schema.definitions.remove(schema.root),
        _ => unreachable!("cw-schema currently emits v1 schemas"),
    }
}

fn struct_properties<T: Schemaifier>() -> BTreeMap<Cow<'static, str>, StructProperty> {
    match root_node::<T>().value {
        NodeType::Struct(StructType::Named { properties }) => properties,
        other => panic!("expected named struct schema, got {other:?}"),
    }
}

fn tuple_item_count<T: Schemaifier>() -> usize {
    match root_node::<T>().value {
        NodeType::Struct(StructType::Tuple { items }) => items.len(),
        other => panic!("expected tuple struct schema, got {other:?}"),
    }
}

fn enum_parts<T: Schemaifier>() -> (
    Option<Cow<'static, str>>,
    BTreeMap<Cow<'static, str>, cw_schema::EnumCase>,
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
    cases: &'a BTreeMap<Cow<'static, str>, cw_schema::EnumCase>,
    name: &str,
) -> &'a BTreeMap<Cow<'static, str>, StructProperty> {
    match &cases.get(name).unwrap().value {
        EnumValue::Named { properties } => properties,
        other => panic!("expected named enum case, got {other:?}"),
    }
}

fn assert_flattening(
    properties: &BTreeMap<Cow<'static, str>, StructProperty>,
    name: &str,
    expected: bool,
) {
    assert_eq!(properties.get(name).unwrap().flattening, expected);
}
