use askama::Template;
use cw_schema_codegen::rust::template::{
    EnumTemplate, EnumVariantTemplate, FieldTemplate, StructTemplate, TypeTemplate,
};
use std::borrow::Cow;

#[test]
fn simple_enum() {
    let tpl = EnumTemplate {
        add_allow: true,
        name: Cow::Borrowed("Simple"),
        docs: Cow::Borrowed(&[Cow::Borrowed("Simple enum")]),
        variants: Cow::Borrowed(&[
            EnumVariantTemplate {
                name: Cow::Borrowed("One"),
                docs: Cow::Borrowed(&[Cow::Borrowed("One variant")]),
                serde_rename: None,
                ty: TypeTemplate::Unit,
            },
            EnumVariantTemplate {
                name: Cow::Borrowed("Two"),
                docs: Cow::Borrowed(&[Cow::Borrowed("Two variant")]),
                serde_rename: None,
                ty: TypeTemplate::Unit,
            },
        ]),
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn complex_enum() {
    let tpl = EnumTemplate {
        add_allow: true,
        name: Cow::Borrowed("Complex"),
        docs: Cow::Borrowed(&[Cow::Borrowed("Complex enum")]),
        variants: Cow::Borrowed(&[
            EnumVariantTemplate {
                name: Cow::Borrowed("One"),
                docs: Cow::Borrowed(&[Cow::Borrowed("One variant")]),
                serde_rename: None,
                ty: TypeTemplate::Tuple(Cow::Borrowed(&[Cow::Borrowed("u64")])),
            },
            EnumVariantTemplate {
                name: Cow::Borrowed("Two"),
                docs: Cow::Borrowed(&[Cow::Borrowed("Two variant")]),
                serde_rename: None,
                ty: TypeTemplate::Named {
                    fields: Cow::Borrowed(&[
                        FieldTemplate {
                            name: Cow::Borrowed("a"),
                            defaulting: false,
                            docs: Cow::Borrowed(&[Cow::Borrowed("Field a")]),
                            ty: Cow::Borrowed("u64"),
                        },
                        FieldTemplate {
                            name: Cow::Borrowed("b"),
                            defaulting: true,
                            docs: Cow::Borrowed(&[Cow::Borrowed("Field b")]),
                            ty: Cow::Borrowed("String"),
                        },
                    ]),
                },
            },
        ]),
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn empty_enum() {
    let tpl = EnumTemplate {
        add_allow: true,
        name: Cow::Borrowed("Empty"),
        docs: Cow::Borrowed(&[Cow::Borrowed("Empty enum")]),
        variants: Cow::Borrowed(&[]),
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn empty_struct() {
    let tpl = StructTemplate {
        add_allow: true,
        name: Cow::Borrowed("Empty"),
        docs: Cow::Borrowed(&[Cow::Borrowed("Empty struct")]),
        ty: TypeTemplate::Unit,
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn tuple_struct() {
    let tpl = StructTemplate {
        add_allow: true,
        name: Cow::Borrowed("Tuple"),
        docs: Cow::Borrowed(&[Cow::Borrowed("Tuple struct")]),
        ty: TypeTemplate::Tuple(Cow::Borrowed(&[
            Cow::Borrowed("u64"),
            Cow::Borrowed("String"),
        ])),
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn named_struct() {
    let tpl = StructTemplate {
        add_allow: true,
        name: Cow::Borrowed("Named"),
        docs: Cow::Borrowed(&[Cow::Borrowed("Named struct")]),
        ty: TypeTemplate::Named {
            fields: Cow::Borrowed(&[
                FieldTemplate {
                    name: Cow::Borrowed("a"),
                    defaulting: false,
                    docs: Cow::Borrowed(&[Cow::Borrowed("Field a")]),
                    ty: Cow::Borrowed("u64"),
                },
                FieldTemplate {
                    name: Cow::Borrowed("b"),
                    defaulting: true,
                    docs: Cow::Borrowed(&[Cow::Borrowed("Field b")]),
                    ty: Cow::Borrowed("String"),
                },
            ]),
        },
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}
