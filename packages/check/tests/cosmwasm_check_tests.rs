use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

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
        .stdout(predicate::str::contains("Deserialization error"));

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
        .stdout(predicate::str::contains("Deserialization error"))
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
