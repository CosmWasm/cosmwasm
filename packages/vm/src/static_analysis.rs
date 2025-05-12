use std::collections::HashSet;

use strum::{AsRefStr, Display, EnumString};
use wasmer::wasmparser::ExternalKind;

use crate::parsed_wasm::ParsedWasm;

/// An enum containing all available contract entrypoints.
/// This also provides conversions to and from strings.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, EnumString, Display, AsRefStr)]
pub enum Entrypoint {
    #[strum(serialize = "instantiate")]
    Instantiate,
    #[strum(serialize = "execute")]
    Execute,
    #[strum(serialize = "migrate")]
    Migrate,
    #[strum(serialize = "sudo")]
    Sudo,
    #[strum(serialize = "reply")]
    Reply,
    #[strum(serialize = "query")]
    Query,
    #[strum(serialize = "ibc_channel_open")]
    IbcChannelOpen,
    #[strum(serialize = "ibc_channel_connect")]
    IbcChannelConnect,
    #[strum(serialize = "ibc_channel_close")]
    IbcChannelClose,
    #[strum(serialize = "ibc_packet_receive")]
    IbcPacketReceive,
    #[strum(serialize = "ibc_packet_ack")]
    IbcPacketAck,
    #[strum(serialize = "ibc_packet_timeout")]
    IbcPacketTimeout,
    #[strum(serialize = "ibc_source_callback")]
    IbcSourceCallback,
    #[strum(serialize = "ibc_destination_callback")]
    IbcDestinationCallback,
    #[strum(serialize = "ibc2_packet_receive")]
    Ibc2PacketReceive,
    #[strum(serialize = "ibc2_packet_timeout")]
    Ibc2PacketTimeout,
    #[strum(serialize = "ibc2_packet_send")]
    Ibc2PacketSend,
}

// sort entrypoints by their &str representation
impl PartialOrd for Entrypoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Entrypoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

pub const REQUIRED_IBC_EXPORTS: &[Entrypoint] = &[
    Entrypoint::IbcChannelOpen,
    Entrypoint::IbcChannelConnect,
    Entrypoint::IbcChannelClose,
    Entrypoint::IbcPacketReceive,
    Entrypoint::IbcPacketAck,
    Entrypoint::IbcPacketTimeout,
];

/// A trait that allows accessing shared functionality of `parity_wasm::elements::Module`
/// and `wasmer::Module` in a shared fashion.
pub trait ExportInfo {
    /// Returns all exported function names with the given prefix
    fn exported_function_names(self, prefix: Option<&str>) -> HashSet<String>;
}

impl ExportInfo for &ParsedWasm<'_> {
    fn exported_function_names(self, prefix: Option<&str>) -> HashSet<String> {
        self.exports
            .iter()
            .filter_map(|export| match export.kind {
                ExternalKind::Func => Some(export.name),
                _ => None,
            })
            .filter(|name| {
                if let Some(required_prefix) = prefix {
                    name.starts_with(required_prefix)
                } else {
                    true
                }
            })
            .map(|name| name.to_string())
            .collect()
    }
}

impl ExportInfo for &wasmer::Module {
    fn exported_function_names(self, prefix: Option<&str>) -> HashSet<String> {
        self.exports()
            .functions()
            .filter_map(|function_export| {
                let name = function_export.name();
                if let Some(required_prefix) = prefix {
                    if name.starts_with(required_prefix) {
                        Some(name.to_string())
                    } else {
                        None
                    }
                } else {
                    Some(name.to_string())
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::wasm_backend::make_compiler_config;
    use crate::VmError;

    use super::*;
    use wasmer::Store;

    static HACKATOM: &[u8] = include_bytes!("../testdata/hackatom.wasm");
    static CORRUPTED: &[u8] = include_bytes!("../testdata/corrupted.wasm");

    #[test]
    fn deserialize_exports_works() {
        let module = ParsedWasm::parse(HACKATOM).unwrap();
        assert_eq!(module.version, 1);

        let exported_functions = module
            .exports
            .iter()
            .filter(|entry| matches!(entry.kind, ExternalKind::Func));
        assert_eq!(exported_functions.count(), 15);

        let exported_memories = module
            .exports
            .iter()
            .filter(|entry| matches!(entry.kind, ExternalKind::Memory));
        assert_eq!(exported_memories.count(), 1);
    }

    #[test]
    fn deserialize_wasm_corrupted_data() {
        match ParsedWasm::parse(CORRUPTED)
            .and_then(|mut parsed| parsed.validate_funcs())
            .unwrap_err()
        {
            VmError::StaticValidationErr { msg, .. } => {
                assert!(msg.starts_with("Wasm bytecode could not be deserialized."))
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn exported_function_names_works_for_parity_with_no_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
            )"#,
        )
        .unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["foo".to_string(), "bar".to_string()])
        );
    }

    #[test]
    fn exported_function_names_works_for_parity_with_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
                (export "baz" (func 0))
            )"#,
        )
        .unwrap();
        let module = ParsedWasm::parse(&wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["bar".to_string(), "baz".to_string()])
        );
    }

    #[test]
    fn exported_function_names_works_for_wasmer_with_no_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let compiler = make_compiler_config();
        let store = Store::new(compiler);
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
            )"#,
        )
        .unwrap();
        let compiler = make_compiler_config();
        let store = Store::new(compiler);
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["foo".to_string(), "bar".to_string()])
        );
    }

    #[test]
    fn exported_function_names_works_for_wasmer_with_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let compiler = make_compiler_config();
        let store = Store::new(compiler);
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
                (export "baz" (func 0))
            )"#,
        )
        .unwrap();
        let compiler = make_compiler_config();
        let store = Store::new(compiler);
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["bar".to_string(), "baz".to_string()])
        );
    }

    #[test]
    fn entrypoint_from_string_works() {
        assert_eq!(
            Entrypoint::from_str("ibc_channel_open").unwrap(),
            Entrypoint::IbcChannelOpen
        );

        assert!(Entrypoint::from_str("IbcChannelConnect").is_err());
    }

    #[test]
    fn entrypoint_to_string_works() {
        assert_eq!(
            Entrypoint::IbcPacketTimeout.to_string(),
            "ibc_packet_timeout"
        );

        let static_str: &'static str = Entrypoint::IbcPacketReceive.as_ref();
        assert_eq!(static_str, "ibc_packet_receive");
    }
}
