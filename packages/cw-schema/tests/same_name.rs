mod module1 {
    #[derive(cw_schema::Schemaifier)]
    pub struct Test {
        foo: usize,
    }
}

mod module2 {
    #[derive(cw_schema::Schemaifier)]
    pub struct Test {
        bar: f32,
    }
}

#[derive(cw_schema::Schemaifier)]
struct Combined {
    module1: module1::Test,
    module2: module2::Test,
}

#[test]
fn can_handle_same_name_in_different_modules() {
    let schema = cw_schema::schema_of::<Combined>();
    insta::assert_json_snapshot!(schema);
}
