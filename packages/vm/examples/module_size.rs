use std::fs::File;
use std::io::Read;
use std::mem;

use clap::{App, Arg};

use cosmwasm_vm::internals::compile_and_use;
use cosmwasm_vm::internals::make_runtime_store;
use cosmwasm_vm::Size;
use wasmer::Module;

pub fn main() {
    let matches = App::new("Module size estimation")
        .version("0.0.2")
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
    let memory_limit = Some(Size::mebi(10));
    let module = compile_and_use(&wasm, memory_limit).unwrap();
    mem::drop(wasm);

    let serialized = module.serialize().unwrap();
    mem::drop(module);

    let module = module_deserialize(&serialized, memory_limit);
    mem::drop(serialized);

    // Report (serialized) module size (again)
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
fn module_deserialize(serialized: &Vec<u8>, memory_limit: Option<Size>) -> Module {
    // Deserialize using make_runtime_store()
    let store = make_runtime_store(memory_limit);
    unsafe { Module::deserialize(&store, serialized) }.unwrap()
}
