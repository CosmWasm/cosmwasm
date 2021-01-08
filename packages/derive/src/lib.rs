#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use std::str::FromStr;

#[proc_macro_attribute]
pub fn entry_point(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    //println!("attr: \"{}\"", attr.to_string());    //println!("item: \"{}\"", item.to_string());
    let cloned = item.clone();
    let function = parse_macro_input!(cloned as syn::ItemFn);
    let name = format_ident!("{}", &function.sig.ident);
    // println!("name: \"{}\"", &name);

    let new_code = format!(
        r##"
        #[cfg(target_arch = "wasm32")]
        mod __wasm_export_{name} {{ // new module to avoid conflict of function name
            #[no_mangle]
            extern "C" fn {name}(env_ptr: u32, info_ptr: u32, msg_ptr: u32) -> u32 {{
                cosmwasm_std::do_{name}(&super::{name}, env_ptr, info_ptr, msg_ptr)
            }}
        }}
    "##,
        name = name,
    );
    let entry = TokenStream::from_str(&new_code).unwrap();
    item.extend(entry);
    item
}
