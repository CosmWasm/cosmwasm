use cargo_manifest::{Dependency, DependencyDetail};
use proc_macro2::TokenStream;
use quote::quote;

use std::env;

use super::bail;

pub fn read_wasmer_version_impl(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
        bail!(input, "unexpected parameters");
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest =
        cargo_manifest::Manifest::from_path(format!("{manifest_dir}/Cargo.toml")).unwrap();

    let Some(dependencies) = &manifest.dependencies else {
        bail!("No dependencies found in Cargo.toml");
    };

    let Some(wasmer_dep) = dependencies.get("wasmer") else {
        bail!("No wasmer dependency found in Cargo.toml");
    };

    let version = match wasmer_dep {
        Dependency::Detailed(DependencyDetail {
            version: Some(version),
            ..
        }) => version,
        Dependency::Simple(version) => version,
        _ => {
            bail!("Wasmer dependency does not have a version");
        }
    };

    quote! { #version }
}
