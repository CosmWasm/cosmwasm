use assert_cmd::prelude::*;
use cosmwasm_std::{to_json_string, to_json_vec};
use cosmwasm_vm::WasmLimits;
use predicates::prelude::*;
use std::{io::Write, process::Command};
use tempfile::NamedTempFile;

#[test]
fn valid_contract_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cosmwasm-check")?;

    cmd.arg("../vm/testdata/hackatom.wasm");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pass"));

    Ok(())
}

#[test]
fn contract_check_verbose() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cosmwasm-check")?;

    cmd.arg("../vm/testdata/empty.wasm").arg("--verbose");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pass"))
        .stderr(predicate::str::contains("Max function parameters"));

    Ok(())
}

#[test]
fn empty_contract_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cosmwasm-check")?;

    cmd.arg("../vm/testdata/empty.wasm");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pass"));

    Ok(())
}

#[test]
fn invalid_contract_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cosmwasm-check")?;

    cmd.arg("../vm/testdata/corrupted.wasm");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("missing a required marker export"));

    Ok(())
}

#[test]
fn valid_contract_check_float_operator() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cosmwasm-check")?;

    cmd.arg("../vm/testdata/floaty.wasm");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pass"));

    Ok(())
}

#[test]
fn several_contracts_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cosmwasm-check")?;

    cmd.arg("../vm/testdata/hackatom.wasm")
        .arg("../vm/testdata/corrupted.wasm");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("failure"))
        .stdout(predicate::str::contains("missing a required marker export"))
        .stdout(predicate::str::contains("Passes: 1, failures: 1"));

    Ok(())
}

#[test]
fn custom_capabilities_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cosmwasm-check")?;

    cmd.arg("--available-capabilities")
        .arg("iterator,osmosis,friendship")
        .arg("../vm/testdata/hackatom.wasm");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available capabilities:"))
        .stdout(predicate::str::contains("iterator"))
        .stdout(predicate::str::contains("osmosis"))
        .stdout(predicate::str::contains("friendship"));

    Ok(())
}

#[test]
fn wasm_limits_base64_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cosmwasm-check")?;

    let mut limits = WasmLimits::default();
    limits.initial_memory_limit = Some(10);

    cmd.arg("--wasm-limits")
        .arg(to_json_string(&limits).unwrap())
        .arg("../vm/testdata/hackatom.wasm");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("must not exceed 10 pages"));

    Ok(())
}

#[test]
fn wasm_limits_file_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cosmwasm-check")?;

    let mut limits = WasmLimits::default();
    limits.max_functions = Some(15);
    let limits = to_json_vec(&limits)?;

    let mut tmp_file = NamedTempFile::new()?;
    tmp_file.write_all(&limits)?;

    cmd.arg("--wasm-limits")
        .arg(tmp_file.path())
        .arg("../vm/testdata/hackatom.wasm");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("more than 15 functions"));

    Ok(())
}
