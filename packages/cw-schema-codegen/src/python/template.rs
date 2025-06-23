use askama::Template;
use std::borrow::Cow;

#[derive(Clone)]
pub struct EnumVariantTemplate<'a> {
    pub name: Cow<'a, str>,
    pub docs: Cow<'a, [Cow<'a, str>]>,
    pub ty: TypeTemplate<'a>,
}

#[derive(Template)]
#[template(escape = "none", path = "python/enum.tpl.py")]
pub struct EnumTemplate<'a> {
    pub name: Cow<'a, str>,
    pub docs: Cow<'a, [Cow<'a, str>]>,
    pub variants: Cow<'a, [EnumVariantTemplate<'a>]>,
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
#[template(escape = "none", path = "python/struct.tpl.py")]
pub struct StructTemplate<'a> {
    pub name: Cow<'a, str>,
    pub docs: Cow<'a, [Cow<'a, str>]>,
    pub ty: TypeTemplate<'a>,
}
