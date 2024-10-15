use askama::Template;

pub struct EnumVariantTemplate<'a> {
    pub name: &'a str,
    pub ty: TypeTemplate<'a>,
}

#[derive(Template)]
#[template(escape = "none", path = "rust/enum.tpl.rs")]
pub struct EnumTemplate<'a> {
    pub name: &'a str,
    pub variants: &'a [EnumVariantTemplate<'a>],
}

pub struct FieldTemplate<'a> {
    pub name: &'a str,
    pub ty: &'a str,
}

pub enum TypeTemplate<'a> {
    Unit,
    Tuple(&'a [&'a str]),
    Named {
        fields: &'a [FieldTemplate<'a>],
    }
}

#[derive(Template)]
#[template(escape = "none", path = "rust/struct.tpl.rs")]
pub struct StructTemplate<'a> {
    pub name: &'a str,
    pub ty: TypeTemplate<'a>,
}
