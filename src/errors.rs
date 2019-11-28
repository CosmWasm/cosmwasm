use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Received null pointer, refuse to use"))]
    NullPointer {
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Error parsing {}: {}", kind, source))]
    ParseErr {
        source: serde_json_wasm::de::Error,
        kind: &'static str,
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Error serializing {}: {}", kind, source))]
    SerializeErr {
        source: serde_json_wasm::ser::Error,
        kind: &'static str,
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Contract error: {}", msg))]
    ContractErr {
        msg: &'static str,
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Contract error: {}", msg))]
    DynContractErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Unauthorized"))]
    Unauthorized {
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
