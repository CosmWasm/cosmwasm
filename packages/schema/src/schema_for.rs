/// Generates a [`RootSchema`](crate::schemars::schema::RootSchema) for the given type using default settings.
///
/// The type must implement [`JsonSchema`](crate::schemars::JsonSchema).
///
/// The schema version is strictly `draft-07`.
///
/// # Example
/// ```
/// use cosmwasm_schema::schema_for;
/// use schemars::JsonSchema;
///
/// #[derive(JsonSchema)]
/// struct MyStruct {
///     foo: i32,
/// }
///
/// let my_schema = schema_for!(MyStruct);
/// ```
#[macro_export]
macro_rules! schema_for {
    ($type:ty) => {
        $crate::schemars::gen::SchemaGenerator::new($crate::schemars::gen::SchemaSettings::draft07()).into_root_schema_for::<$type>()
    };
    ($_:expr) => {
        compile_error!("The argument to `schema_for!` is not a type.")
    };
}
