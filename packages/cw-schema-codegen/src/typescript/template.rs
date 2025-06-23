use askama::Template;
use std::borrow::Cow;

#[derive(Clone)]
pub struct EnumVariantTemplate<'a> {
    pub name: Cow<'a, str>,
    pub docs: Cow<'a, [Cow<'a, str>]>,
    pub ty: TypeTemplate<'a>,
}

#[derive(Template)]
#[template(escape = "none", path = "typescript/enum.tpl.ts")]
pub struct EnumTemplate<'a> {
    pub name: Cow<'a, str>,
    pub docs: Cow<'a, [Cow<'a, str>]>,
    pub variants: Cow<'a, [EnumVariantTemplate<'a>]>,
    pub add_imports: bool,
}

#[derive(Clone)]
pub struct FieldTemplate<'a> {
    pub name: Cow<'a, str>,
    pub docs: Cow<'a, [Cow<'a, str>]>,
    pub ty: Cow<'a, str>,
}

#[derive(Clone)]
pub enum TypeTemplate<'a> {
    Unit,
    Tuple(Cow<'a, [Cow<'a, str>]>),
    Named {
        fields: Cow<'a, [FieldTemplate<'a>]>,
    },
}

#[derive(Template)]
#[template(escape = "none", path = "typescript/struct.tpl.ts")]
pub struct StructTemplate<'a> {
    pub name: Cow<'a, str>,
    pub docs: Cow<'a, [Cow<'a, str>]>,
    pub ty: TypeTemplate<'a>,
    pub add_imports: bool,
}
