use askama::Template;

#[derive(Template)]
#[template(escape = "none", path = "typescript/enum.tpl.ts")]
pub struct EnumTemplate {}

#[derive(Template)]
#[template(escape = "none", path = "typescript/struct.tpl.ts")]
pub struct StructTemplate {}
