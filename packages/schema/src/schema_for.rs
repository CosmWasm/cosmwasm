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
    ($type:ty) => {{
        let mut schema = $crate::schemars::gen::SchemaGenerator::new(
            $crate::schemars::gen::SchemaSettings::draft07(),
        )
        .into_root_schema_for::<$type>();

        struct Visitor;
        impl $crate::schemars::visit::Visitor for Visitor {
            fn visit_schema_object(&mut self, schema: &mut $crate::schemars::schema::SchemaObject) {
                $crate::schemars::visit::visit_schema_object(self, schema);

                if let Some(ref mut validation) = schema.object {
                    validation.additional_properties = Some(Box::new(false.into()));
                }
            }
        }

        $crate::schemars::visit::visit_root_schema(&mut Visitor, &mut schema);

        schema
    }};
    ($_:expr) => {
        compile_error!("The argument to `schema_for!` is not a type.")
    };
}
