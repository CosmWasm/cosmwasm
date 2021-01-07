#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2;

#[proc_macro_attribute]
pub fn entry_point(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let cloned = item.clone();
    let ast = parse_macro_input!(cloned as syn::DeriveInput);
    let export_name = format_ident!("{}", &ast.ident);
    let export_impl = format_ident!("{}", &ast.ident);

    let item: proc_macro2::TokenStream = item.into();

    let gen = quote! {
        #item

        #[no_mangle]
        extern "C" fn hahaha_#export_name(env_ptr: u32, info_ptr: u32, msg_ptr: u32) -> u32 {
            do_migrate(&#export_impl, env_ptr, info_ptr, msg_ptr)
        }
    };
    gen.into()
}
