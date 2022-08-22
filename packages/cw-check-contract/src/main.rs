use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use clap::{App, Arg};

use cosmwasm_vm::capabilities_from_csv;
use cosmwasm_vm::internals::{check_wasm, compile};

const DEFAULT_AVAILABLE_CAPABILITIES: &str = "iterator,staking,stargate";

pub fn main() {
    let matches = App::new("Contract checking")
        .version("0.1.0")
        .long_about("Checks the given wasm file (memories, exports, imports, available capabilities, and non-determinism).")
        .author("Mauro Lacy <mauro@lacy.com.es>")
        .arg(
            Arg::with_name("CAPABILITIES")
                // `long` setting required to turn the position argument into an option 🤷
                .long("available-capabilities")
                .aliases(&["FEATURES", "supported-features"]) // Old names
                .value_name("CAPABILITIES")
                .help("Sets the available capabilities that the desired target chain has")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("WASM")
                .help("Wasm file to read and compile")
                .required(true)
                .index(1)
                .multiple(true),
        )
        .get_matches();

    // Available capabilities
    let available_capabilities_csv = matches
        .value_of("CAPABILITIES")
        .unwrap_or(DEFAULT_AVAILABLE_CAPABILITIES);
    let available_capabilities = capabilities_from_csv(available_capabilities_csv);
    println!("Available capabilities: {:?}", available_capabilities);

    // File
    let paths = matches.values_of("WASM").expect("Error parsing file names");

    for path in paths {
        check_contract(path, &available_capabilities).unwrap();
    }

    println!("contract checks passed.")
}

fn check_contract(
    path: impl AsRef<Path>,
    available_capabilities: &HashSet<String>,
) -> anyhow::Result<()> {
    let mut file = File::open(path)?;

    // Read wasm
    let mut wasm = Vec::<u8>::new();
    file.read_to_end(&mut wasm)?;

    // Check wasm
    check_wasm(&wasm, available_capabilities)?;

    // Compile module
    compile(&wasm, None, &[])?;

    Ok(())
}
