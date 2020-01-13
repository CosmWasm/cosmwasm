use wasm_nm::{Options, Symbol, Symbols};

use crate::errors::{Result, ValidationErr};

static PUBLIC_SYMBOLS: Options = Options {
    imports: true,
    exports: true,
    privates: false,
    sizes: false,
};

/// Lists all imports we provide upon instantiating the instance in Instance::from_module()
/// This should be updated when new imports are added
static SUPPORTED_IMPORTS: &[&'static str] = &[
    "c_read",
    "c_write",
    "c_canonical_address",
    "c_human_address",
];

/// Lists all entry points we expect to be present when calling a contract.
/// Basically, anything that is used in calls.rs
/// This is unlikely to change much, must be frozen at 1.0 to avoid breaking existing contracts
static REQUIRED_EXPORTS: &[&'static str] = &["query", "init", "handle", "allocate", "deallocate"];

pub fn check_api_compatibility(wasm_code: &[u8]) -> Result<()> {
    let mut reader = std::io::Cursor::new(wasm_code);
    let symbols = wasm_nm::symbols(PUBLIC_SYMBOLS.clone(), &mut reader).unwrap();
    if !only_imports(&symbols, SUPPORTED_IMPORTS) {
        return ValidationErr {
            msg: "WASM requires unsupported imports - version too new?",
        }
        .fail();
    }
    if !has_all_exports(&symbols, REQUIRED_EXPORTS) {
        return ValidationErr {
            msg: "WASM doesn't have required exports - version too old?",
        }
        .fail();
    }
    Ok(())
}

fn only_imports(symbols: &Symbols, allowed: &[&str]) -> bool {
    let imports: Vec<&str> = symbols
        .iter()
        .filter_map(|s| match s {
            Symbol::Import { name } => Some(name),
            _ => None,
        })
        .collect();

    for i in imports {
        if !allowed.contains(&i) {
            return false;
        }
    }
    true
}

fn has_all_exports(symbols: &Symbols, required: &[&str]) -> bool {
    let exports: Vec<&str> = symbols
        .iter()
        .filter_map(|s| match s {
            Symbol::Export { name, .. } => Some(name),
            _ => None,
        })
        .collect();

    for i in required {
        if !exports.contains(&i) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod test {
    use super::*;

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    #[test]
    fn test_supported_imports() {
        let mut reader = std::io::Cursor::new(CONTRACT);
        let symbols = wasm_nm::symbols(PUBLIC_SYMBOLS.clone(), &mut reader).unwrap();

        // if contract has more than we provide, bad
        let imports_good = only_imports(&symbols, &["c_read", "c_write"]);
        assert_eq!(imports_good, false);

        // exact match good
        let imports_good = only_imports(
            &symbols,
            &[
                "c_read",
                "c_write",
                "c_canonical_address",
                "c_human_address",
            ],
        );
        assert_eq!(imports_good, true);

        // if we provide more, also good
        let imports_good = only_imports(
            &symbols,
            &[
                "c_read",
                "c_write",
                "c_canonical_address",
                "c_human_address",
                "future_function",
            ],
        );
        assert_eq!(imports_good, true);
    }

    #[test]
    fn test_required_exports() {
        let mut reader = std::io::Cursor::new(CONTRACT);
        let symbols = wasm_nm::symbols(PUBLIC_SYMBOLS.clone(), &mut reader).unwrap();

        // subset okay
        let exports_good = has_all_exports(&symbols, &["init", "handle", "allocate"]);
        assert_eq!(exports_good, true);

        // match okay
        let exports_good = has_all_exports(
            &symbols,
            &["query", "init", "handle", "allocate", "deallocate"],
        );
        assert_eq!(exports_good, true);

        // missing one from list not okay
        let exports_good = has_all_exports(&symbols, &["init", "handle", "extra"]);
        assert_eq!(exports_good, false);
    }

    #[test]
    fn test_api_compatibility() {
        // this is our reference check, must pass
        check_api_compatibility(CONTRACT).unwrap();

        // TODO: add some failing examples
    }
}
