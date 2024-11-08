use std::borrow::Cow;

use askama::Template;
use cw_schema_codegen::python::template::{
    EnumTemplate, EnumVariantTemplate, FieldTemplate, StructTemplate, TypeTemplate,
};

#[test]
fn simple_enum() {
    let tpl = EnumTemplate {
        name: Cow::Borrowed("Simple"),
        docs: Cow::Borrowed(&[Cow::Borrowed("Simple enum")]),
        variants: Cow::Borrowed(&[
            EnumVariantTemplate {
                name: Cow::Borrowed("One"),
                docs: Cow::Borrowed(&[Cow::Borrowed("One variant")]),
                ty: TypeTemplate::Unit,
            },
            EnumVariantTemplate {
                name: Cow::Borrowed("Two"),
                docs: Cow::Borrowed(&[Cow::Borrowed("Two variant")]),
                ty: TypeTemplate::Unit,
            },
        ]),
    };

    let rendered = tpl.render().unwrap();
    insta::assert_snapshot!(rendered);
}
