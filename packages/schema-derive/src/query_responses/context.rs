use std::collections::HashSet;

use syn::{Ident, ItemEnum, Meta, NestedMeta};

const ATTR_PATH: &str = "query_responses";

pub struct Context {
    /// If the enum we're trying to derive QueryResponses for collects other QueryMsgs,
    /// setting this flag will derive the implementation appropriately, collecting all
    /// KV pairs from the nested enums rather than expecting `#[return]` annotations.
    pub is_nested: bool,
    /// Disable infering the `JsonSchema` trait bound for chosen type parameters.
    pub no_bounds_for: HashSet<Ident>,
}

pub fn get_context(input: &ItemEnum) -> Context {
    let params = input
        .attrs
        .iter()
        .filter(|attr| matches!(attr.path.get_ident(), Some(id) if *id == ATTR_PATH))
        .flat_map(|attr| {
            if let Meta::List(l) = attr.parse_meta().unwrap() {
                l.nested
            } else {
                panic!("{} attribute must contain a meta list", ATTR_PATH);
            }
        })
        .map(|nested_meta| {
            if let NestedMeta::Meta(m) = nested_meta {
                m
            } else {
                panic!("no literals allowed in QueryResponses params")
            }
        });

    let mut ctx = Context {
        is_nested: false,
        no_bounds_for: HashSet::new(),
    };

    for param in params {
        match param.path().get_ident().unwrap().to_string().as_str() {
            "no_bounds_for" => {
                if let Meta::List(l) = param {
                    for item in l.nested {
                        match item {
                            NestedMeta::Meta(Meta::Path(p)) => {
                                ctx.no_bounds_for.insert(p.get_ident().unwrap().clone());
                            }
                            _ => panic!("`no_bounds_for` only accepts a list of type params"),
                        }
                    }
                } else {
                    panic!("expected a list for `no_bounds_for`")
                }
            }
            "nested" => ctx.is_nested = true,
            path => panic!("unrecognized QueryResponses param: {}", path),
        }
    }

    ctx
}
