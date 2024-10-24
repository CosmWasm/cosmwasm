use askama::Template;
use cw_schema_codegen::rust::{EnumTemplate, EnumVariantTemplate, FieldTemplate, TypeTemplate};

#[test]
fn simple_enum() {
    let tpl = EnumTemplate {
        name: "Simple",
        variants: &[
            EnumVariantTemplate {
                name: "One",
                ty: TypeTemplate::Unit,
            },
            EnumVariantTemplate {
                name: "Two",
                ty: TypeTemplate::Unit,
            },
        ],
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn complex_enum() {
    let tpl = EnumTemplate {
        name: "Complex",
        variants: &[
            EnumVariantTemplate {
                name: "One",
                ty: TypeTemplate::Tuple(&["u64"]),
            },
            EnumVariantTemplate {
                name: "Two",
                ty: TypeTemplate::Named {
                    fields: &[
                        FieldTemplate {
                            name: "a",
                            ty: "u64",
                        },
                        FieldTemplate {
                            name: "b",
                            ty: "String",
                        },
                    ],
                },
            },
        ],
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn empty_enum() {
    let tpl = EnumTemplate {
        name: "Empty",
        variants: &[],
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}
