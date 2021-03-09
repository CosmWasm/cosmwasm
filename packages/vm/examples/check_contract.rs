use std::fs::File;
use std::io::Read;

use clap::{App, Arg};

use cosmwasm_vm::features_from_csv;
use cosmwasm_vm::internals::{check_wasm, compile};

pub fn main() {
    let matches = App::new("Contract checking")
        .version("0.1.0")
        .long_about("Checks the given wasm file (memories, exports, imports, supported features, and non-determinism).")
        .author("Mauro Lacy <mauro@lacy.com.es>")
        .arg(
            Arg::with_name("WASM")
                .help("Wasm file to read and compile")
                .required(true)
                .index(1),
        )
        .get_matches();

    // File
    let path = matches.value_of("WASM").expect("Error parsing file name");
    let mut file = File::open(path).unwrap();

    // Read wasm
    let mut wasm = Vec::<u8>::new();
    file.read_to_end(&mut wasm).unwrap();

    // Check wasm
    check_wasm(&wasm, &features_from_csv("staking,stargate")).unwrap();

    // Compile module
    compile(&wasm, None).unwrap();
    println!("contract checks passed.")
}
