use cw_schema::Schemaifier;

#[derive(Schemaifier)]
/// Hello world struct!
struct HelloWorld {
    /// Name field!
    name: String,

    /// Foo field!
    foo: Option<Bar>,

    /// Baz field!
    baz: Baz,

    /// Tuple field!
    tuple: (u32, u32),
}

#[derive(Schemaifier)]
/// Bar struct!
struct Bar {
    /// Bar field!
    baz: u32,
}

#[derive(Schemaifier)]
/// Baz enum!
enum Baz {
    /// A variant!
    A,
    /// B variant!
    B {
        /// C field!
        c: u32,
    },
    /// D variant!
    D(u32, u32),
}

#[test]
fn snapshot_schema() {
    let schema = cw_schema::schema_of::<HelloWorld>();
    insta::assert_json_snapshot!(schema);
}
