use std::fs::File;
use std::io::Read;

use clap::{App, Arg};

use cosmwasm_vm::capabilities_from_csv;
use cosmwasm_vm::internals::{check_wasm, compile};

const DEFAULT_AVAILABLE_CAPABILITIES: &str = "iterator,staking,stargate";

pub fn main() {
    eprintln!("`check_contract` will be removed from the next version of `cosmwasm-vm` - please use `cw-check-contract` instead.");
    eprintln!("> cargo install cw-check-contract");

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
                .index(1),
        )
        .get_matches();

    // Available capabilities
    let available_capabilities_csv = matches
        .value_of("CAPABILITIES")
        .unwrap_or(DEFAULT_AVAILABLE_CAPABILITIES);
    let available_capabilities = capabilities_from_csv(available_capabilities_csv);
    println!("Available capabilities: {:?}", available_capabilities);

    // File
    let path = matches.value_of("WASM").expect("Error parsing file name");
    let mut file = File::open(path).unwrap();

    // Read wasm
    let mut wasm = Vec::<u8>::new();
    file.read_to_end(&mut wasm).unwrap();

    // Check wasm
    check_wasm(&wasm, &available_capabilities).unwrap();

    // Compile module
    compile(&wasm, None, &[]).unwrap();
    println!("contract checks passed.")
}
