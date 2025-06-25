use askama::Template;
use std::borrow::Cow;

#[derive(Clone)]
pub struct EnumVariantTemplate<'a> {
    pub name: Cow<'a, str>,
    pub rename: Cow<'a, str>,
    pub docs: Cow<'a, [Cow<'a, str>]>,
    pub ty: TypeTemplate<'a>,
}

#[derive(Template)]
#[template(escape = "none", path = "go/enum.tpl.go")]
pub struct EnumTemplate<'a> {
    pub add_package: bool,
    pub name: Cow<'a, str>,
    pub docs: Cow<'a, [Cow<'a, str>]>,
    pub variants: Cow<'a, [EnumVariantTemplate<'a>]>,
    pub has_unit_variants: bool,
}

#[derive(Clone)]
pub struct FieldTemplate<'a> {
    pub name: Cow<'a, str>,
    pub rename: Cow<'a, str>,
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

impl TypeTemplate<'_> {
    pub fn is_unit(&self) -> bool {
        matches!(self, TypeTemplate::Unit)
    }
}

#[derive(Template)]
#[template(escape = "none", path = "go/struct.tpl.go")]
pub struct StructTemplate<'a> {
    pub add_package: bool,
    pub name: Cow<'a, str>,
    pub docs: Cow<'a, [Cow<'a, str>]>,
    pub ty: TypeTemplate<'a>,
}
