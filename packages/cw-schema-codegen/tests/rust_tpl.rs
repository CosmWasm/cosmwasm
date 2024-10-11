use askama::Template;
use cw_schema_codegen::rust::{EnumTemplate, EnumVariantTemplate};

#[test]
fn simple_enum() {
    let tpl = EnumTemplate {
        name: "Simple",
        variants: &[
            EnumVariantTemplate {
                name: "One",
                types: None,
            },
            EnumVariantTemplate {
                name: "Two",
                types: None,
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
                types: Some(&["u64"]),
            },
            EnumVariantTemplate {
                name: "Two",
                types: Some(&["String", "u64"]),
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
