#![deny(unsafe_code)]
#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pednatic)]
#![allow(forbidden_lint_groups)]

pub mod api;
pub mod db;
pub mod error;
pub mod iterator;
pub mod metrics;
pub mod querier;

uniffi::setup_scaffolding!();

#[derive(uniffi::Record)]
pub struct GasMeter {}

#[derive(uniffi::Record)]
pub struct AnalysisReport {
    pub has_ibc_entry_points: bool,
    pub entrypoints: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub contract_migrate_version: Option<u64>,
}

#[derive(uniffi::Record)]
pub struct GasReport {
    pub limit: u64,
    pub remaining: u64,
    pub used_externally: u64,
    pub used_internally: u64,
}

#[uniffi::export]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").into()
}
