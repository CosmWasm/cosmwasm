mod context;

use crate::error::{bail, error_message};
use syn::{
    parse_quote, Expr, ExprTuple, Generics, ItemEnum, ItemImpl, Type, TypeParamBound, Variant,
};

use self::context::Context;

enum SchemaBackend {
    CwSchema,
    JsonSchema,
}

pub fn query_responses_derive_impl(input: ItemEnum) -> syn::Result<ItemImpl> {
    let ctx = context::get_context(&input)?;

    let item_impl = if ctx.is_nested {
        let crate_name = &ctx.crate_name;
        let ident = input.ident;
        let subquery_calls = input
            .variants
            .iter()
            .map(|variant| parse_subquery(&ctx, variant, SchemaBackend::JsonSchema))
            .collect::<syn::Result<Vec<_>>>()?;

        let subquery_calls_cw = input
            .variants
            .iter()
            .map(|variant| parse_subquery(&ctx, variant, SchemaBackend::CwSchema))
            .collect::<syn::Result<Vec<_>>>()?;

        // Handle generics if the type has any
        let (_, type_generics, where_clause) = input.generics.split_for_impl();
        let impl_generics = impl_generics(
            &ctx,
            &input.generics,
            &[parse_quote! {#crate_name::QueryResponses}],
        );

        let subquery_len = subquery_calls.len();
        parse_quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl #impl_generics #crate_name::QueryResponses for #ident #type_generics #where_clause {
                fn response_schemas_impl() -> ::std::collections::BTreeMap<String, #crate_name::schemars::schema::RootSchema> {
                    let subqueries = [
                        #( #subquery_calls, )*
                    ];
                    #crate_name::combine_subqueries::<#subquery_len, #ident #type_generics, _>(subqueries)
                }

                fn response_schemas_cw_impl() -> ::std::collections::BTreeMap<String, #crate_name::cw_schema::Schema> {
                    let subqueries = [
                        #( #subquery_calls_cw, )*
                    ];
                    #crate_name::combine_subqueries::<#subquery_len, #ident #type_generics, _>(subqueries)
                }
            }
        }
    } else {
        let crate_name = &ctx.crate_name;
        let ident = input.ident;
        let mappings = input
            .variants
            .iter()
            .map(|variant| parse_query(&ctx, variant, SchemaBackend::JsonSchema))
            .collect::<syn::Result<Vec<_>>>()?;

        let mappings = mappings.into_iter().map(parse_tuple);

        let cw_mappings = input
            .variants
            .iter()
            .map(|variant| parse_query(&ctx, variant, SchemaBackend::CwSchema))
            .collect::<syn::Result<Vec<_>>>()?;

        let cw_mappings = cw_mappings.into_iter().map(parse_tuple);

        // Handle generics if the type has any
        let (_, type_generics, where_clause) = input.generics.split_for_impl();
        let impl_generics = impl_generics(&ctx, &input.generics, &[]);

        parse_quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl #impl_generics #crate_name::QueryResponses for #ident #type_generics #where_clause {
                fn response_schemas_impl() -> ::std::collections::BTreeMap<String, #crate_name::schemars::schema::RootSchema> {
                    ::std::collections::BTreeMap::from([
                        #( #mappings, )*
                    ])
                }

                fn response_schemas_cw_impl() -> ::std::collections::BTreeMap<String, #crate_name::cw_schema::Schema> {
                    ::std::collections::BTreeMap::from([
                        #( #cw_mappings, )*
                    ])
                }
            }
        }
    };
    Ok(item_impl)
}

/// Takes a list of generics from the type definition and produces a list of generics
/// for the expanded `impl` block, adding trait bounds like `JsonSchema` as appropriate.
fn impl_generics(ctx: &Context, generics: &Generics, bounds: &[TypeParamBound]) -> Generics {
    let mut impl_generics = generics.to_owned();
    for param in impl_generics.type_params_mut() {
        // remove the default type if present, as those are invalid in
        // a trait implementation
        param.default = None;

        if !ctx.no_bounds_for.contains(&param.ident) {
            let crate_name = &ctx.crate_name;

            param
                .bounds
                .push(parse_quote! {#crate_name::schemars::JsonSchema});
            param
                .bounds
                .push(parse_quote! { #crate_name::cw_schema::Schemaifier });

            param.bounds.extend(bounds.to_owned());
        }
    }

    impl_generics
}

/// Extract the query -> response mapping out of an enum variant.
fn parse_query(
    ctx: &Context,
    v: &Variant,
    schema_backend: SchemaBackend,
) -> syn::Result<(String, Expr)> {
    let crate_name = &ctx.crate_name;
    let query = to_snake_case(&v.ident.to_string());
    let response_ty: Type = v
        .attrs
        .iter()
        .find(|a| a.path().is_ident("returns"))
        .ok_or_else(|| error_message!(&v, "missing return type for query"))?
        .parse_args()
        .map_err(|e| error_message!(e.span(), "return must be a type"))?;

    let return_val = match schema_backend {
        SchemaBackend::CwSchema => {
            parse_quote!(#crate_name::cw_schema::schema_of::<#response_ty>())
        }
        SchemaBackend::JsonSchema => parse_quote!(#crate_name::schema_for!(#response_ty)),
    };

    Ok((query, return_val))
}

/// Extract the nested query  -> response mapping out of an enum variant.
fn parse_subquery(ctx: &Context, v: &Variant, schema_backend: SchemaBackend) -> syn::Result<Expr> {
    let crate_name = &ctx.crate_name;
    let submsg = match v.fields {
        syn::Fields::Named(_) => bail!(v, "a struct variant is not a valid subquery"),
        syn::Fields::Unnamed(ref fields) => {
            if fields.unnamed.len() != 1 {
                bail!(fields, "invalid number of subquery parameters");
            }

            &fields.unnamed[0].ty
        }
        syn::Fields::Unit => bail!(v, "a unit variant is not a valid subquery"),
    };

    let return_val = match schema_backend {
        SchemaBackend::CwSchema => {
            parse_quote!(<#submsg as #crate_name::QueryResponses>::response_schemas_cw_impl())
        }
        SchemaBackend::JsonSchema => {
            parse_quote!(<#submsg as #crate_name::QueryResponses>::response_schemas_impl())
        }
    };

    Ok(return_val)
}

fn parse_tuple((q, r): (String, Expr)) -> ExprTuple {
    parse_quote! {
        (#q.to_string(), #r)
    }
}

fn to_snake_case(input: &str) -> String {
    // this was stolen from serde for consistent behavior
    let mut snake = String::new();
    for (i, ch) in input.char_indices() {
        if i > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use syn::parse_quote;

    use super::*;

    fn test_context() -> Context {
        Context {
            crate_name: parse_quote!(::cosmwasm_schema),
            is_nested: false,
            no_bounds_for: HashSet::new(),
        }
    }

    #[test]
    fn crate_rename() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(rename_all = "snake_case")]
            #[query_responses(crate = "::my_crate::cw_schema")]
            pub enum QueryMsg {
                #[returns(some_crate::AnotherType)]
                Supply {},
                #[returns(SomeType)]
                Balance {},
            }
        };

        assert_eq!(
            query_responses_derive_impl(input).unwrap(),
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl ::my_crate::cw_schema::QueryResponses for QueryMsg {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::my_crate::cw_schema::schemars::schema::RootSchema> {
                        ::std::collections::BTreeMap::from([
                            ("supply".to_string(), ::my_crate::cw_schema::schema_for!(some_crate::AnotherType)),
                            ("balance".to_string(), ::my_crate::cw_schema::schema_for!(SomeType)),
                        ])
                    }

                    fn response_schemas_cw_impl() -> ::std::collections::BTreeMap<String, ::my_crate::cw_schema::cw_schema::Schema> {
                        ::std::collections::BTreeMap::from([
                            ("supply".to_string(), ::my_crate::cw_schema::cw_schema::schema_of::<some_crate::AnotherType>()),
                            ("balance".to_string(), ::my_crate::cw_schema::cw_schema::schema_of::<SomeType>()),
                        ])
                    }
                }
            }
        );
    }

    #[test]
    fn crate_rename_nested() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(untagged)]
            #[query_responses(crate = "::my_crate::cw_schema", nested)]
            pub enum ContractQueryMsg {
                Cw1(QueryMsg1),
                Whitelist(whitelist::QueryMsg),
                Cw1WhitelistContract(QueryMsg),
            }
        };
        let result = query_responses_derive_impl(input).unwrap();
        assert_eq!(
            result,
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl ::my_crate::cw_schema::QueryResponses for ContractQueryMsg {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::my_crate::cw_schema::schemars::schema::RootSchema> {
                        let subqueries = [
                            <QueryMsg1 as ::my_crate::cw_schema::QueryResponses>::response_schemas_impl(),
                            <whitelist::QueryMsg as ::my_crate::cw_schema::QueryResponses>::response_schemas_impl(),
                            <QueryMsg as ::my_crate::cw_schema::QueryResponses>::response_schemas_impl(),
                        ];
                        ::my_crate::cw_schema::combine_subqueries::<3usize, ContractQueryMsg, _>(subqueries)
                    }

                    fn response_schemas_cw_impl() -> ::std::collections::BTreeMap<String, ::my_crate::cw_schema::cw_schema::Schema> {
                        let subqueries = [
                            <QueryMsg1 as ::my_crate::cw_schema::QueryResponses>::response_schemas_cw_impl(),
                            <whitelist::QueryMsg as ::my_crate::cw_schema::QueryResponses>::response_schemas_cw_impl(),
                            <QueryMsg as ::my_crate::cw_schema::QueryResponses>::response_schemas_cw_impl(),
                        ];
                        ::my_crate::cw_schema::combine_subqueries::<3usize, ContractQueryMsg, _>(subqueries)
                    }
                }
            }
        );
    }

    #[test]
    fn happy_path() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(rename_all = "snake_case")]
            pub enum QueryMsg {
                #[returns(some_crate::AnotherType)]
                Supply {},
                #[returns(SomeType)]
                Balance {},
            }
        };

        assert_eq!(
            query_responses_derive_impl(input).unwrap(),
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl ::cosmwasm_schema::QueryResponses for QueryMsg {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                        ::std::collections::BTreeMap::from([
                            ("supply".to_string(), ::cosmwasm_schema::schema_for!(some_crate::AnotherType)),
                            ("balance".to_string(), ::cosmwasm_schema::schema_for!(SomeType)),
                        ])
                    }

                    fn response_schemas_cw_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::cw_schema::Schema> {
                        ::std::collections::BTreeMap::from([
                            ("supply".to_string(), ::cosmwasm_schema::cw_schema::schema_of::<some_crate::AnotherType>()),
                            ("balance".to_string(), ::cosmwasm_schema::cw_schema::schema_of::<SomeType>()),
                        ])
                    }
                }
            }
        );
    }

    #[test]
    fn empty_query_msg() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(rename_all = "snake_case")]
            pub enum QueryMsg {}
        };

        assert_eq!(
            query_responses_derive_impl(input).unwrap(),
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl ::cosmwasm_schema::QueryResponses for QueryMsg {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                        ::std::collections::BTreeMap::from([])
                    }

                    fn response_schemas_cw_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::cw_schema::Schema> {
                        ::std::collections::BTreeMap::from([])
                    }
                }
            }
        );
    }

    #[test]
    fn generics() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(rename_all = "snake_case")]
            pub enum QueryMsg<T> {
                #[returns(bool)]
                Foo,
                #[returns(u32)]
                Bar(T),
            }
        };

        let input2: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(rename_all = "snake_case")]
            pub enum QueryMsg<T: std::fmt::Debug + SomeTrait> {
                #[returns(bool)]
                Foo,
                #[returns(u32)]
                Bar { data: T },
            }
        };

        let input3: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(rename_all = "snake_case")]
            pub enum QueryMsg<T>
                where T: std::fmt::Debug + SomeTrait,
            {
                #[returns(bool)]
                Foo,
                #[returns(u32)]
                Bar { data: T },
            }
        };

        let result = query_responses_derive_impl(input).unwrap();

        assert_eq!(
            result,
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl<T: ::cosmwasm_schema::schemars::JsonSchema + ::cosmwasm_schema::cw_schema::Schemaifier> ::cosmwasm_schema::QueryResponses for QueryMsg<T> {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                        ::std::collections::BTreeMap::from([
                            ("foo".to_string(), ::cosmwasm_schema::schema_for!(bool)),
                            ("bar".to_string(), ::cosmwasm_schema::schema_for!(u32)),
                        ])
                    }

                    fn response_schemas_cw_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::cw_schema::Schema> {
                        ::std::collections::BTreeMap::from([
                            ("foo".to_string(), ::cosmwasm_schema::cw_schema::schema_of::<bool>()),
                            ("bar".to_string(), ::cosmwasm_schema::cw_schema::schema_of::<u32>()),
                        ])
                    }
                }
            }
        );
        assert_eq!(
            query_responses_derive_impl(input2).unwrap(),
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl<T: std::fmt::Debug + SomeTrait + ::cosmwasm_schema::schemars::JsonSchema + ::cosmwasm_schema::cw_schema::Schemaifier> ::cosmwasm_schema::QueryResponses for QueryMsg<T> {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                        ::std::collections::BTreeMap::from([
                            ("foo".to_string(), ::cosmwasm_schema::schema_for!(bool)),
                            ("bar".to_string(), ::cosmwasm_schema::schema_for!(u32)),
                        ])
                    }

                    fn response_schemas_cw_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::cw_schema::Schema> {
                        ::std::collections::BTreeMap::from([
                            ("foo".to_string(), ::cosmwasm_schema::cw_schema::schema_of::<bool>()),
                            ("bar".to_string(), ::cosmwasm_schema::cw_schema::schema_of::<u32>()),
                        ])
                    }
                }
            }
        );
        let a = query_responses_derive_impl(input3).unwrap();
        assert_eq!(
            a,
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl<T: ::cosmwasm_schema::schemars::JsonSchema + ::cosmwasm_schema::cw_schema::Schemaifier> ::cosmwasm_schema::QueryResponses for QueryMsg<T>
                    where T: std::fmt::Debug + SomeTrait,
                {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                        ::std::collections::BTreeMap::from([
                            ("foo".to_string(), ::cosmwasm_schema::schema_for!(bool)),
                            ("bar".to_string(), ::cosmwasm_schema::schema_for!(u32)),
                        ])
                    }

                    fn response_schemas_cw_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::cw_schema::Schema> {
                        ::std::collections::BTreeMap::from([
                            ("foo".to_string(), ::cosmwasm_schema::cw_schema::schema_of::<bool>()),
                            ("bar".to_string(), ::cosmwasm_schema::cw_schema::schema_of::<u32>()),
                        ])
                    }
                }
            }
        );
    }

    #[test]
    #[should_panic(expected = "missing return type for query")]
    fn missing_return() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(rename_all = "snake_case")]
            pub enum QueryMsg {
                Supply {},
                #[returns(SomeType)]
                Balance {},
            }
        };

        query_responses_derive_impl(input).unwrap();
    }

    #[test]
    #[should_panic(expected = "return must be a type")]
    fn invalid_return() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(rename_all = "snake_case")]
            pub enum QueryMsg {
                #[returns(1)]
                Supply {},
                #[returns(SomeType)]
                Balance {},
            }
        };

        query_responses_derive_impl(input).unwrap();
    }

    #[test]
    fn parse_query_works() {
        let variant = parse_quote! {
            #[returns(Foo)]
            GetFoo {}
        };

        assert_eq!(
            parse_tuple(parse_query(&test_context(), &variant, SchemaBackend::JsonSchema).unwrap()),
            parse_quote! {
                ("get_foo".to_string(), ::cosmwasm_schema::schema_for!(Foo))
            }
        );
        assert_eq!(
            parse_tuple(parse_query(&test_context(), &variant, SchemaBackend::CwSchema).unwrap()),
            parse_quote! {
                ("get_foo".to_string(), ::cosmwasm_schema::cw_schema::schema_of::<Foo>())
            }
        );

        let variant = parse_quote! {
            #[returns(some_crate::Foo)]
            GetFoo {}
        };

        assert_eq!(
            parse_tuple(parse_query(&test_context(), &variant, SchemaBackend::JsonSchema).unwrap()),
            parse_quote! { ("get_foo".to_string(), ::cosmwasm_schema::schema_for!(some_crate::Foo)) }
        );
        assert_eq!(
            parse_tuple(parse_query(&test_context(), &variant, SchemaBackend::CwSchema).unwrap()),
            parse_quote! { ("get_foo".to_string(), ::cosmwasm_schema::cw_schema::schema_of::<some_crate::Foo>()) }
        );
    }

    #[test]
    fn to_snake_case_works() {
        assert_eq!(to_snake_case("SnakeCase"), "snake_case");
        assert_eq!(to_snake_case("Wasm123AndCo"), "wasm123_and_co");
    }

    #[test]
    fn nested_works() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(untagged)]
            #[query_responses(nested)]
            pub enum ContractQueryMsg {
                Cw1(QueryMsg1),
                Whitelist(whitelist::QueryMsg),
                Cw1WhitelistContract(QueryMsg),
            }
        };
        let result = query_responses_derive_impl(input).unwrap();
        assert_eq!(
            result,
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl ::cosmwasm_schema::QueryResponses for ContractQueryMsg {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                        let subqueries = [
                            <QueryMsg1 as ::cosmwasm_schema::QueryResponses>::response_schemas_impl(),
                            <whitelist::QueryMsg as ::cosmwasm_schema::QueryResponses>::response_schemas_impl(),
                            <QueryMsg as ::cosmwasm_schema::QueryResponses>::response_schemas_impl(),
                        ];
                        ::cosmwasm_schema::combine_subqueries::<3usize, ContractQueryMsg, _>(subqueries)
                    }

                    fn response_schemas_cw_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::cw_schema::Schema> {
                        let subqueries = [
                            <QueryMsg1 as ::cosmwasm_schema::QueryResponses>::response_schemas_cw_impl(),
                            <whitelist::QueryMsg as ::cosmwasm_schema::QueryResponses>::response_schemas_cw_impl(),
                            <QueryMsg as ::cosmwasm_schema::QueryResponses>::response_schemas_cw_impl(),
                        ];
                        ::cosmwasm_schema::combine_subqueries::<3usize, ContractQueryMsg, _>(subqueries)
                    }
                }
            }
        );
    }

    #[test]
    fn nested_empty() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(untagged)]
            #[query_responses(nested)]
            pub enum EmptyMsg {}
        };
        let result = query_responses_derive_impl(input).unwrap();
        assert_eq!(
            result,
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl ::cosmwasm_schema::QueryResponses for EmptyMsg {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                        let subqueries = [];
                        ::cosmwasm_schema::combine_subqueries::<0usize, EmptyMsg, _>(subqueries)
                    }

                    fn response_schemas_cw_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::cw_schema::Schema> {
                        let subqueries = [];
                        ::cosmwasm_schema::combine_subqueries::<0usize, EmptyMsg, _>(subqueries)
                    }
                }
            }
        );
    }

    #[test]
    #[should_panic(expected = "invalid number of subquery parameters")]
    fn nested_too_many_params() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(untagged)]
            #[query_responses(nested)]
            pub enum ContractQueryMsg {
                Msg1(QueryMsg1, QueryMsg2),
                Whitelist(whitelist::QueryMsg),
            }
        };
        query_responses_derive_impl(input).unwrap();
    }

    #[test]
    #[should_panic(expected = "a struct variant is not a valid subquery")]
    fn nested_mixed() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(untagged)]
            #[query_responses(nested)]
            pub enum ContractQueryMsg {
                Cw1(cw1::QueryMsg),
                Test {
                    mixed: bool,
                }
            }
        };
        query_responses_derive_impl(input).unwrap();
    }

    #[test]
    #[should_panic(expected = "a unit variant is not a valid subquery")]
    fn nested_unit_variant() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(untagged)]
            #[query_responses(nested)]
            pub enum ContractQueryMsg {
                Cw1(cw1::QueryMsg),
                Whitelist,
            }
        };
        query_responses_derive_impl(input).unwrap();
    }
}
