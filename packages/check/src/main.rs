use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;

use clap::{Arg, ArgAction, Command};
use colored::Colorize;

use cosmwasm_vm::capabilities_from_csv;
use cosmwasm_vm::internals::{check_wasm, compile, make_compiling_engine};

const DEFAULT_AVAILABLE_CAPABILITIES: &str =
    "iterator,staking,stargate,cosmwasm_1_1,cosmwasm_1_2,cosmwasm_1_3";

pub fn main() {
    let matches = Command::new("Contract checking")
        .version(env!("CARGO_PKG_VERSION"))
        .long_about("Checks the given wasm file (memories, exports, imports, available capabilities, and non-determinism).")
        .author("Mauro Lacy <mauro@lacy.com.es>")
        .arg(
            Arg::new("CAPABILITIES")
                // `long` setting required to turn the position argument into an option 🤷
                .long("available-capabilities")
                .aliases(["FEATURES", "supported-features"]) // Old names
                .value_name("CAPABILITIES")
                .help("Sets the available capabilities that the desired target chain has")
                .num_args(1)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("WASM")
                .help("Wasm file to read and compile")
                .required(true)
                .index(1)
                .num_args(0..)
                .action(ArgAction::Append),
        )
        .get_matches();

    // Available capabilities
    let available_capabilities_csv = matches
        .get_one::<String>("CAPABILITIES")
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT_AVAILABLE_CAPABILITIES);
    let available_capabilities = capabilities_from_csv(available_capabilities_csv);
    println!("Available capabilities: {available_capabilities:?}");
    println!();

    // File
    let paths = matches
        .get_many::<String>("WASM")
        .expect("Error parsing file names");

    let (passes, failures): (Vec<_>, _) = paths
        .map(|p| {
            let result = check_contract(p, &available_capabilities);
            match &result {
                Ok(_) => println!("{}: {}", p, "pass".green()),
                Err(e) => {
                    println!("{}: {}", p, "failure".red());
                    println!("{e}");
                }
            };
            result
        })
        .partition(|result| result.is_ok());
    println!();

    if failures.is_empty() {
        println!(
            "All contracts ({}) {} checks!",
            passes.len(),
            "passed".green()
        );
    } else {
        println!(
            "{}: {}, {}: {}",
            "Passes".green(),
            passes.len(),
            "failures".red(),
            failures.len()
        );
        exit(1);
    }
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
    let engine = make_compiling_engine(None);
    let _module = compile(&engine, &wasm)?;

    Ok(())
}
