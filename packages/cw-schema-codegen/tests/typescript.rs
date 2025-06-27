use core::str;
use std::{fs::File, io::Write, process::Command};

use crate::utils::TestCase;

mod utils;

#[test]
fn codegen_snap() {
    // generate the schemas for each of the above types
    let schemas = [
        cw_schema::schema_of::<utils::Owo>(),
        cw_schema::schema_of::<utils::Uwu>(),
        cw_schema::schema_of::<utils::Òwó>(),
        cw_schema::schema_of::<utils::Empty>(),
        cw_schema::schema_of::<utils::Hehehe>(),
    ];

    // run the codegen to typescript
    for schema in schemas {
        let cw_schema::Schema::V1(schema) = schema else {
            panic!();
        };

        let output = schema
            .definitions
            .iter()
            .map(|node| {
                let mut buf = Vec::new();
                cw_schema_codegen::typescript::process_node(&mut buf, &schema, node, true).unwrap();
                String::from_utf8(buf).unwrap()
            })
            .collect::<String>();

        insta::assert_snapshot!(output);
    }
}

#[test]
#[ignore] // because it requires NPM to be installed, CI will still run it
fn assert_validity() {
    let e2e_dir = format!("{}/tests/ts-e2e", env!("CARGO_MANIFEST_DIR"));
    let gen_file_path = format!("{}/src/gen.ts", e2e_dir);

    // make sure the dependencies are installed
    let install_status = Command::new("npm")
        .arg("i")
        .current_dir(&e2e_dir)
        .status()
        .unwrap();
    assert!(install_status.success());

    utils::run_e2e(
        |buf, schema, node| cw_schema_codegen::typescript::process_node(buf, schema, node, true),
        |TestCase { code, type_name }| {
            let mut gen_file = File::create(&gen_file_path).unwrap();
            gen_file.write_all(code.as_bytes()).unwrap();

            let mut cmd = Command::new("npm");
            cmd.args(["test".into(), format!("{type_name}Schema")])
                .current_dir(&e2e_dir);

            cmd
        },
    );
}
