use std::collections::HashSet;

use crate::error::bail;
use syn::{parse_quote, Ident, ItemEnum, LitStr};

const ATTR_PATH: &str = "query_responses";

pub struct Context {
    /// Name of the crate referenced in the macro expansions
    pub crate_name: syn::Path,
    /// If the enum we're trying to derive QueryResponses for collects other QueryMsgs,
    /// setting this flag will derive the implementation appropriately, collecting all
    /// KV pairs from the nested enums rather than expecting `#[return]` annotations.
    pub is_nested: bool,
    /// Disable inferring the `JsonSchema` trait bound for chosen type parameters.
    pub no_bounds_for: HashSet<Ident>,
}

pub fn get_context(input: &ItemEnum) -> syn::Result<Context> {
    let mut ctx = Context {
        crate_name: parse_quote!(::cosmwasm_schema),
        is_nested: false,
        no_bounds_for: HashSet::new(),
    };

    for attr in &input.attrs {
        if !attr.path().is_ident(ATTR_PATH) {
            continue;
        }

        let meta_list = attr.meta.require_list()?;
        meta_list.parse_nested_meta(|param| {
            if param.path.is_ident("no_bounds_for") {
                param.parse_nested_meta(|item| {
                    ctx.no_bounds_for
                        .insert(item.path.get_ident().unwrap().clone());

                    Ok(())
                })?;
            } else if param.path.is_ident("nested") {
                ctx.is_nested = true;
            } else if param.path.is_ident("crate") {
                let crate_name_str: LitStr = param.value()?.parse()?;
                ctx.crate_name = crate_name_str.parse()?;
            } else {
                bail!(param.path, "unrecognized QueryResponses param");
            }

            Ok(())
        })?;
    }

    Ok(ctx)
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use quote::format_ident;
    use syn::parse_quote;

    use super::get_context;

    #[test]
    fn parse_context() {
        let input = parse_quote! {
            #[query_responses(crate = "::my_crate::cw_schema")]
            #[query_responses(nested)]
            #[query_responses(no_bounds_for(Item1, Item2))]
            enum Test {}
        };
        let context = get_context(&input).unwrap();

        assert_eq!(context.crate_name, parse_quote!(::my_crate::cw_schema));
        assert!(context.is_nested);
        assert_eq!(
            context.no_bounds_for,
            HashSet::from([format_ident!("Item1"), format_ident!("Item2")])
        );
    }
}
