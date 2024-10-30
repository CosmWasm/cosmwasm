use askama::Template;

#[derive(Template)]
#[template(escape = "none", path = "go/enum.tpl.go")]
pub struct EnumTemplate {}

#[derive(Template)]
#[template(escape = "none", path = "go/struct.tpl.go")]
pub struct StructTemplate {}
