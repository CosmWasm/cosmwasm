use syn::{parse_quote, Expr, ExprTuple, Ident, ItemEnum, ItemImpl, Type, Variant};

pub fn query_responses_derive_impl(input: ItemEnum) -> ItemImpl {
    let is_nested = has_attr(&input, "query_responses", "nested");

    if is_nested {
        if !has_attr(&input, "serde", "untagged") {
            panic!("#[query_responses(nested)] only makes sense in conjunction with #[serde(untagged)]");
        }
        let ident = input.ident;
        let subquery_calls = input.variants.into_iter().map(parse_subquery);

        // Handle generics if the type has any
        let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

        let subquery_len = subquery_calls.len();
        parse_quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl #impl_generics ::cosmwasm_schema::QueryResponses for #ident #type_generics #where_clause {
                fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                    let subqueries = [
                        #( #subquery_calls, )*
                    ];
                    ::cosmwasm_schema::combine_subqueries::<#subquery_len, #ident>(subqueries)
                }
            }
        }
    } else {
        let ident = input.ident;
        let mappings = input.variants.into_iter().map(parse_query);
        let mut queries: Vec<_> = mappings.clone().map(|(q, _)| q).collect();
        queries.sort();
        let mappings = mappings.map(parse_tuple);

        // Handle generics if the type has any
        let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

        parse_quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl #impl_generics ::cosmwasm_schema::QueryResponses for #ident #type_generics #where_clause {
                fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                    ::std::collections::BTreeMap::from([
                        #( #mappings, )*
                    ])
                }
            }
        }
    }
}

/// Extract the query -> response mapping out of an enum variant.
fn parse_query(v: Variant) -> (String, Expr) {
    let query = to_snake_case(&v.ident.to_string());
    let response_ty: Type = v
        .attrs
        .iter()
        .find(|a| a.path.get_ident().unwrap() == "returns")
        .unwrap_or_else(|| panic!("missing return type for query: {}", v.ident))
        .parse_args()
        .unwrap_or_else(|_| panic!("return for {} must be a type", v.ident));

    (
        query,
        parse_quote!(::cosmwasm_schema::schema_for!(#response_ty)),
    )
}

/// Extract the nested query  -> response mapping out of an enum variant.
fn parse_subquery(v: Variant) -> Expr {
    let submsg = match v.fields {
        syn::Fields::Named(_) => panic!("nested query response can only contain tuple structs"),
        syn::Fields::Unnamed(fields) => {
            if fields.unnamed.len() != 1 {
                panic!("invalid number of subquery parameters");
            }

            fields.unnamed[0].ty.clone()
        }
        syn::Fields::Unit => panic!("a unit variant is not a valid subquery"),
    };
    parse_quote!(<#submsg as ::cosmwasm_schema::QueryResponses>::response_schemas_impl())
}

/// Checks whether the input has the given #[query_responses($attr))] attribute
fn has_attr(input: &ItemEnum, path: &str, attr: &str) -> bool {
    input.attrs.iter().any(|a| {
        a.path.get_ident().unwrap() == path
            && a.parse_args::<Ident>().ok().map_or(false, |i| i == attr)
    })
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
    use syn::parse_quote;

    use super::*;

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
            query_responses_derive_impl(input),
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
            query_responses_derive_impl(input),
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl ::cosmwasm_schema::QueryResponses for QueryMsg {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
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

        let result = query_responses_derive_impl(input);
        dbg!(&result);
        assert_eq!(
            result,
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl<T> ::cosmwasm_schema::QueryResponses for QueryMsg<T> {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                        ::std::collections::BTreeMap::from([
                            ("foo".to_string(), ::cosmwasm_schema::schema_for!(bool)),
                            ("bar".to_string(), ::cosmwasm_schema::schema_for!(u32)),
                        ])
                    }
                }
            }
        );
        assert_eq!(
            query_responses_derive_impl(input2),
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl<T: std::fmt::Debug + SomeTrait> ::cosmwasm_schema::QueryResponses for QueryMsg<T> {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                        ::std::collections::BTreeMap::from([
                            ("foo".to_string(), ::cosmwasm_schema::schema_for!(bool)),
                            ("bar".to_string(), ::cosmwasm_schema::schema_for!(u32)),
                        ])
                    }
                }
            }
        );
        let a = query_responses_derive_impl(input3);
        assert_eq!(
            a,
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl<T> ::cosmwasm_schema::QueryResponses for QueryMsg<T>
                    where T: std::fmt::Debug + SomeTrait,
                {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                        ::std::collections::BTreeMap::from([
                            ("foo".to_string(), ::cosmwasm_schema::schema_for!(bool)),
                            ("bar".to_string(), ::cosmwasm_schema::schema_for!(u32)),
                        ])
                    }
                }
            }
        );
    }

    #[test]
    #[should_panic(expected = "missing return type for query: Supply")]
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

        query_responses_derive_impl(input);
    }

    #[test]
    #[should_panic(expected = "return for Supply must be a type")]
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

        query_responses_derive_impl(input);
    }

    #[test]
    fn parse_query_works() {
        let variant = parse_quote! {
            #[returns(Foo)]
            GetFoo {}
        };

        assert_eq!(
            parse_tuple(parse_query(variant)),
            parse_quote! {
                ("get_foo".to_string(), ::cosmwasm_schema::schema_for!(Foo))
            }
        );

        let variant = parse_quote! {
            #[returns(some_crate::Foo)]
            GetFoo {}
        };

        assert_eq!(
            parse_tuple(parse_query(variant)),
            parse_quote! { ("get_foo".to_string(), ::cosmwasm_schema::schema_for!(some_crate::Foo)) }
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
        let result = query_responses_derive_impl(input);
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
                        ::cosmwasm_schema::combine_subqueries::<3usize, ContractQueryMsg>(subqueries)
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
        let result = query_responses_derive_impl(input);
        assert_eq!(
            result,
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl ::cosmwasm_schema::QueryResponses for EmptyMsg {
                    fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::cosmwasm_schema::schemars::schema::RootSchema> {
                        let subqueries = [];
                        ::cosmwasm_schema::combine_subqueries::<0usize, EmptyMsg>(subqueries)
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
        query_responses_derive_impl(input);
    }

    #[test]
    #[should_panic(expected = "nested query response can only contain tuple structs")]
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
        query_responses_derive_impl(input);
    }

    #[test]
    #[should_panic(
        expected = "#[query_responses(nested)] only makes sense in conjunction with #[serde(untagged)]"
    )]
    fn nested_without_untagged() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[query_responses(nested)]
            pub enum ContractQueryMsg {
                Cw1(cw1::QueryMsg),
            }
        };
        query_responses_derive_impl(input);
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
        query_responses_derive_impl(input);
    }
}
