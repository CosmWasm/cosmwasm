use std::{fs::File, io::Write, process::Command};

use crate::utils::TestCase;

mod utils;

#[test]
fn e2e() {
    let e2e_dir = format!("{}/tests/rust-e2e", env!("CARGO_MANIFEST_DIR"));
    let gen_file_path = format!("{e2e_dir}/src/gen.rs");

    utils::run_e2e(
        |buf, schema, node| cw_schema_codegen::rust::process_node(buf, schema, node, true),
        |TestCase { code, type_name }| {
            let mut gen_file = File::create(&gen_file_path).unwrap();
            gen_file.write_all(code.as_bytes()).unwrap();
            gen_file
                .write_all(format!("pub type TestType = {type_name};").as_bytes())
                .unwrap();

            let mut cmd = Command::new("cargo");
            cmd.args(["run"]).current_dir(&e2e_dir);

            cmd
        },
    );
}
