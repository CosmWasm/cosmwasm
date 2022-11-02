#[macro_export]
macro_rules! schema_for {
    ($type:ty) => {
        $crate::schemars::gen::SchemaGenerator::new($crate::schemars::gen::SchemaSettings::draft07()).into_root_schema_for::<$type>()
    };
    ($_:expr) => {
        compile_error!("This argument to `schema_for!` is not a type - did you mean to use `schema_for_value!` instead?")
    };
}
