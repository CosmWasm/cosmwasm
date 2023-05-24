use std::fs::File;
use std::io::Read;
use std::mem;

use clap::{App, Arg};

use cosmwasm_vm::internals::compile;
use cosmwasm_vm::internals::make_engine;
use wasmer::Module;

pub fn main() {
    let matches = App::new("Module size estimation")
        .version("0.0.4")
        .author("Mauro Lacy <mauro@confio.gmbh>")
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

    // Report wasm size
    let wasm_size = wasm.len();
    println!("wasm size: {} bytes", wasm_size);

    // Compile module
    let module = module_compile(&wasm);
    mem::drop(wasm);

    let serialized = module.serialize().unwrap();
    mem::drop(module);

    // Deserialize module
    let module = module_deserialize(&serialized);
    mem::drop(serialized);

    // Report (serialized) module size
    let serialized = module.serialize().unwrap();
    mem::drop(module);
    let ser_size = serialized.len();
    println!("module size (serialized): {} bytes", ser_size);
    println!(
        "(serialized) module size ratio: {:.2}",
        ser_size as f32 / wasm_size as f32
    );
}

#[inline(never)]
fn module_compile(wasm: &[u8]) -> Module {
    let (_engine, module) = compile(wasm, &[]).unwrap();
    module
}

#[inline(never)]
fn module_deserialize(serialized: &[u8]) -> Module {
    // Deserialize using make_engine()
    let engine = make_engine(&[]);
    unsafe { Module::deserialize(&engine, serialized) }.unwrap()
}
