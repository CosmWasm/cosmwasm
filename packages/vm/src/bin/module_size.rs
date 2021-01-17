use std::fs::File;
use std::io::Read;
use std::mem;

use clap::{App, Arg};

use cosmwasm_vm::compile_only;

pub fn main() {
    let matches = App::new("Module size estimation")
        .version("0.0.1")
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
    mem::drop(matches);

    // Read wasm
    let mut wasm = Vec::<u8>::new();
    file.read_to_end(&mut wasm).unwrap();
    mem::drop(file);

    // Report size
    let wasm_size = wasm.len();
    println!("wasm size: {} bytes", wasm_size);

    // Compile module
    let module = compile_only(&wasm).unwrap();
    mem::drop(wasm);

    // Report (serialized) module size
    let ser_size = module.serialize().unwrap().len();
    println!("module size (serialized): {} bytes", ser_size);
    println!(
        "(serialized) module size ratio: {:.2}",
        ser_size as f32 / wasm_size as f32
    );
}
