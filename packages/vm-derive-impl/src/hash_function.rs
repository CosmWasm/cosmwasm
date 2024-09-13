use std::{
    io::Write,
    process::{Command, Stdio},
};

use proc_macro2::TokenStream;
use quote::quote;

use super::{bail, maybe};

// i do what i must because i can <https://youtu.be/Y6ljFaKRTrI?t=27>
fn format_code<C>(code: C) -> String
where
    C: AsRef<[u8]>,
{
    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(code.as_ref()).unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

pub fn hash_function_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        bail!(attr, "Unexpected parameters");
    }

    // Just verify that this is actually a function
    let _: syn::ItemFn = maybe!(syn::parse2(input.clone()));

    let display = format_code(input.to_string());
    let hex_hash = blake3::hash(display.as_bytes()).to_hex();
    let hex_hash = hex_hash.as_str();

    quote! {
        #input

        ::cosmwasm_vm_derive::inventory::submit! {
            ::cosmwasm_vm_derive::Hash(#hex_hash)
        }
    }
}
