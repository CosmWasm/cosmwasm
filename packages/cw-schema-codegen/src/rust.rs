use askama::Template;

pub struct EnumVariantTemplate<'a> {
    pub name: &'a str,
    pub types: Option<&'a [&'a str]>,
}

#[derive(Template)]
#[template(escape = "none", path = "rust/enum.tpl.rs")]
pub struct EnumTemplate<'a> {
    pub name: &'a str,
    pub variants: &'a [EnumVariantTemplate<'a>],
}
