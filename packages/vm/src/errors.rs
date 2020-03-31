use std::fmt::Debug;
use std::io;

use snafu::Snafu;
use wasmer_runtime_core::cache::Error as CacheError;
use wasmer_runtime_core::error as core_error;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Cache error: {}", msg))]
    CacheErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Couldn't convert from {} to {}. Input: {}", from_type, to_type, input))]
    ConversionErr {
        from_type: &'static str,
        to_type: &'static str,
        input: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Compiling wasm: {}", source))]
    CompileErr {
        source: core_error::CompileError,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Filesystem error: {}", source))]
    IoErr {
        source: io::Error,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Hash doesn't match stored data"))]
    IntegrityErr { backtrace: snafu::Backtrace },
    #[snafu(display("Parse error: {}", source))]
    ParseErr {
        kind: &'static str,
        source: serde_json_wasm::de::Error,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Serialize error: {}", source))]
    SerializeErr {
        kind: &'static str,
        source: serde_json_wasm::ser::Error,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Resolving wasm function: {}", source))]
    ResolveErr {
        source: core_error::ResolveError,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Calling wasm function: {}", source))]
    RuntimeErr {
        source: core_error::RuntimeError,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Region too small. Got {}, required {}", size, required))]
    RegionTooSmallErr {
        size: usize,
        required: usize,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Validating Wasm: {}", msg))]
    ValidationErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Wasmer error: {}", source))]
    WasmerErr {
        source: core_error::Error,
        backtrace: snafu::Backtrace,
    },
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub trait CacheExt<T: Debug> {
    fn convert_cache(self) -> Result<T>;
}

impl<T: Debug> CacheExt<T> for Result<T, CacheError> {
    fn convert_cache(self) -> Result<T> {
        self.map_err(|err| {
            let msg = format!("{:?}", err);
            // construct like this (not just Err(Error::CacheErr)) to allow backtraces
            let res: Result<T> = CacheErr { msg }.fail();
            res.unwrap_err()
        })
    }
}
