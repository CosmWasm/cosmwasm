use std::fs::File;
use std::io::Read;

use clap::{App, Arg};

use cosmwasm_vm::internals::compile;

pub fn main() {
    let matches = App::new("Module compilation")
        .version("0.1.0")
        .long_about("Checks that the given wasm file compiles successfully")
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

    // Compile module
    compile(&wasm, None).unwrap();
    println!("module compiled successfully")
}
