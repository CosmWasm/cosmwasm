use assert_cmd::{cargo_bin, cmd::Command};
use cosmwasm_std::{to_json_string, to_json_vec};
use cosmwasm_vm::WasmLimits;
use predicates::{boolean::PredicateBooleanExt, str::contains};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn valid_contract_check() {
    Command::new(cargo_bin!())
        .arg("../vm/testdata/hackatom.wasm")
        .assert()
        .success()
        .stdout(contains("pass"));
}

#[test]
fn contract_check_verbose() {
    Command::new(cargo_bin!())
        .arg("../vm/testdata/empty.wasm")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(contains("pass"))
        .stderr(contains("Max function parameters"));
}

#[test]
fn empty_contract_check() {
    Command::new(cargo_bin!())
        .arg("../vm/testdata/empty.wasm")
        .assert()
        .success()
        .stdout(contains("pass"));
}

#[test]
fn invalid_contract_check() {
    Command::new(cargo_bin!())
        .arg("../vm/testdata/corrupted.wasm")
        .assert()
        .failure()
        .stdout(contains("missing a required marker export"));
}

#[test]
fn valid_contract_check_float_operator() {
    Command::new(cargo_bin!())
        .arg("../vm/testdata/floaty.wasm")
        .assert()
        .success()
        .stdout(contains("pass"));
}

#[test]
fn several_contracts_check() {
    Command::new(cargo_bin!())
        .arg("../vm/testdata/hackatom.wasm")
        .arg("../vm/testdata/corrupted.wasm")
        .assert()
        .failure()
        .stdout(
            contains("failure")
                .and(contains("missing a required marker export"))
                .and(contains("Passes: 1, failures: 1")),
        );
}

#[test]
fn custom_capabilities_check() {
    Command::new(cargo_bin!())
    .arg("--available-capabilities")
        .arg("iterator,osmosis,friendship,cosmwasm_1_1,cosmwasm_1_2,cosmwasm_1_3,cosmwasm_1_4,cosmwasm_2_0,cosmwasm_2_1,cosmwasm_2_2")
        .arg("../vm/testdata/hackatom.wasm")
    .assert().success().stdout(
        contains("Available capabilities:")
            .and(contains("iterator"))
            .and(contains("osmosis"))
            .and(contains("friendship")),
    );
}

#[test]
fn wasm_limits_string_check() {
    let mut limits = WasmLimits::default();
    limits.initial_memory_limit_pages = Some(10);
    let limits = to_json_string(&limits).unwrap();

    Command::new(cargo_bin!())
        .arg("--wasm-limits")
        .arg(limits)
        .arg("../vm/testdata/hackatom.wasm")
        .assert()
        .failure()
        .stdout(contains("must not exceed 10 pages"));
}

#[test]
fn wasm_limits_file_check() {
    let mut limits = WasmLimits::default();
    limits.max_functions = Some(15);
    let limits = to_json_vec(&limits).unwrap();

    let mut tmp_file = NamedTempFile::new().unwrap();
    tmp_file.write_all(&limits).unwrap();

    Command::new(cargo_bin!())
        .arg("--wasm-limits")
        .arg(tmp_file.path())
        .arg("../vm/testdata/hackatom.wasm")
        .assert()
        .failure()
        .stdout(contains("more than 15 functions"));
}
