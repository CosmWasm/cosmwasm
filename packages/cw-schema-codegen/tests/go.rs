use std::{fs::File, io::Write, process::Command};

use crate::utils::TestCase;

mod utils;

#[test]
#[ignore] // because it requires Go to be installed, CI will still run it
fn e2e() {
    let e2e_dir = format!("{}/tests/go-e2e", env!("CARGO_MANIFEST_DIR"));
    let gen_file_path = format!("{e2e_dir}/gen.go");

    // make sure the dependencies are installed
    let install_status = Command::new("go")
        .args(["get"])
        .current_dir(&e2e_dir)
        .status()
        .unwrap();

    assert!(install_status.success());

    utils::run_e2e(
        |buf, schema, node| cw_schema_codegen::go::process_node(buf, schema, node, true),
        |TestCase { code, type_name }| {
            let mut gen_file = File::create(&gen_file_path).unwrap();
            gen_file.write_all(b"package main\n").unwrap();
            gen_file.write_all(code.as_bytes()).unwrap();
            gen_file
                .write_all(format!("type TestType = {type_name}").as_bytes())
                .unwrap();

            let mut cmd = Command::new("go");
            cmd.args(["run", "main.go", "gen.go"]).current_dir(&e2e_dir);

            cmd
        },
    );
}
