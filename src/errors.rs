use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
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
    #[snafu(display("{} not found", kind))]
    NotFound {
        kind: &'static str,
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
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
    #[snafu(display("UTF8 encoding error: {}", source))]
    Utf8Err {
        source: std::str::Utf8Error,
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Unauthorized"))]
    Unauthorized {
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Invalid {}: {}", field, msg))]
    ValidationErr {
        field: &'static str,
        msg: &'static str,
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub fn invalid<T>(field: &'static str, msg: &'static str) -> Result<T> {
    ValidationErr { field, msg }.fail()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn use_invalid() {
        let e: Result<()> = invalid("demo", "not implemented");
        match e {
            Err(Error::ValidationErr { field, msg, .. }) => {
                assert_eq!(field, "demo");
                assert_eq!(msg, "not implemented");
            }
            Err(e) => panic!("unexpected error, {:?}", e),
            Ok(_) => panic!("invalid must return error"),
        }
    }
}
