use std::collections::HashSet;

use manyhow::bail;
use syn::{Ident, ItemEnum};

const ATTR_PATH: &str = "query_responses";

pub struct Context {
    /// If the enum we're trying to derive QueryResponses for collects other QueryMsgs,
    /// setting this flag will derive the implementation appropriately, collecting all
    /// KV pairs from the nested enums rather than expecting `#[return]` annotations.
    pub is_nested: bool,
    /// Disable infering the `JsonSchema` trait bound for chosen type parameters.
    pub no_bounds_for: HashSet<Ident>,
}

pub fn get_context(input: &ItemEnum) -> syn::Result<Context> {
    let mut ctx = Context {
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
                let meta_list: syn::MetaList = param.input.parse()?;
                meta_list.parse_nested_meta(|item| {
                    let syn::Meta::Path(p) = item.input.parse()? else {
                        bail!(
                            item.input.span(),
                            "`no_bounds_for` only accepts a list of type params"
                        );
                    };

                    ctx.no_bounds_for.insert(p.get_ident().unwrap().clone());

                    Ok(())
                })?;
            } else if param.path.is_ident("nested") {
                ctx.is_nested = true;
            } else {
                bail!(param.path, "unrecognized QueryResponses param");
            }

            Ok(())
        })?;
    }

    Ok(ctx)
}
