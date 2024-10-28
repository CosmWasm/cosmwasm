use std::borrow::Cow;

use askama::Template;
use cw_schema_codegen::rust::template::{
    EnumTemplate, EnumVariantTemplate, FieldTemplate, StructTemplate, TypeTemplate,
};

#[test]
fn simple_enum() {
    let tpl = EnumTemplate {
        name: "Simple",
        docs: Cow::Borrowed(&[Cow::Borrowed("Simple enum")]),
        variants: Cow::Borrowed(&[
            EnumVariantTemplate {
                name: "One",
                docs: Cow::Borrowed(&[Cow::Borrowed("One variant")]),
                ty: TypeTemplate::Unit,
            },
            EnumVariantTemplate {
                name: "Two",
                docs: Cow::Borrowed(&[Cow::Borrowed("Two variant")]),
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
        name: "Complex",
        docs: Cow::Borrowed(&[Cow::Borrowed("Complex enum")]),
        variants: Cow::Borrowed(&[
            EnumVariantTemplate {
                name: "One",
                docs: Cow::Borrowed(&[Cow::Borrowed("One variant")]),
                ty: TypeTemplate::Tuple(Cow::Borrowed(&[Cow::Borrowed("u64")])),
            },
            EnumVariantTemplate {
                name: "Two",
                docs: Cow::Borrowed(&[Cow::Borrowed("Two variant")]),
                ty: TypeTemplate::Named {
                    fields: Cow::Borrowed(&[
                        FieldTemplate {
                            name: Cow::Borrowed("a"),
                            docs: Cow::Borrowed(&[Cow::Borrowed("Field a")]),
                            ty: Cow::Borrowed("u64"),
                        },
                        FieldTemplate {
                            name: Cow::Borrowed("b"),
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
        name: "Empty",
        docs: Cow::Borrowed(&[Cow::Borrowed("Empty enum")]),
        variants: Cow::Borrowed(&[]),
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn empty_struct() {
    let tpl = StructTemplate {
        name: "Empty",
        docs: Cow::Borrowed(&[Cow::Borrowed("Empty struct")]),
        ty: TypeTemplate::Unit,
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn tuple_struct() {
    let tpl = StructTemplate {
        name: "Tuple",
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
        name: "Named",
        docs: Cow::Borrowed(&[Cow::Borrowed("Named struct")]),
        ty: TypeTemplate::Named {
            fields: Cow::Borrowed(&[
                FieldTemplate {
                    name: Cow::Borrowed("a"),
                    docs: Cow::Borrowed(&[Cow::Borrowed("Field a")]),
                    ty: Cow::Borrowed("u64"),
                },
                FieldTemplate {
                    name: Cow::Borrowed("b"),
                    docs: Cow::Borrowed(&[Cow::Borrowed("Field b")]),
                    ty: Cow::Borrowed("String"),
                },
            ]),
        },
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}
