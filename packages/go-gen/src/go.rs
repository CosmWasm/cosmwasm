use std::fmt::{self, Display, Write};

use indenter::indented;
use inflector::Inflector;

use crate::utils::replace_acronyms;

pub struct GoStruct {
    pub name: String,
    pub docs: Option<String>,
    pub fields: Vec<GoField>,
}

impl Display for GoStruct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // generate documentation
        format_docs(f, self.docs.as_deref())?;
        // generate type
        writeln!(f, "type {} struct {{", self.name)?;
        // generate fields
        {
            let mut f = indented(f);
            for field in &self.fields {
                writeln!(f, "{}", field)?;
            }
        }
        f.write_char('}')?;
        Ok(())
    }
}

pub struct GoField {
    /// The name of the field in Rust (snake_case)
    pub rust_name: String,
    /// The documentation of the field
    pub docs: Option<String>,
    /// The type of the field
    pub ty: GoType,
}

impl Display for GoField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // documentation
        format_docs(f, self.docs.as_deref())?;
        // {field} {type} `json:"{field}"`
        write!(
            f,
            "{} {} `json:\"{}",
            replace_acronyms(self.rust_name.to_pascal_case()),
            self.ty,
            self.rust_name
        )?;
        if self.ty.is_nullable {
            f.write_str(",omitempty")?;
        }
        f.write_str("\"`")
    }
}

pub struct GoType {
    /// The name of the type in Go
    pub name: String,
    /// Whether the type should be nullable
    /// This will add `omitempty` to the json tag and use a pointer type if
    /// the type is not a basic type
    pub is_nullable: bool,
}

impl GoType {
    pub fn is_basic_type(&self) -> bool {
        const BASIC_GO_TYPES: &[&str] = &[
            "string",
            "bool",
            "int",
            "int8",
            "int16",
            "int32",
            "int64",
            "uint",
            "uint8",
            "uint16",
            "uint32",
            "uint64",
            "float32",
            "float64",
            "byte",
            "rune",
            "uintptr",
            "complex64",
            "complex128",
        ];
        BASIC_GO_TYPES.contains(&&*self.name)
    }
}

impl Display for GoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_nullable && !self.is_basic_type() {
            // if the type is nullable and not a basic type, use a pointer
            f.write_char('*')?;
        }
        f.write_str(&self.name)
    }
}

fn format_docs(f: &mut fmt::Formatter, docs: Option<&str>) -> fmt::Result {
    if let Some(docs) = docs {
        for line in docs.lines() {
            f.write_str("// ")?;
            f.write_str(line)?;
            f.write_char('\n')?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_type_display_works() {
        let ty = GoType {
            name: "string".to_string(),
            is_nullable: true,
        };
        let ty2 = GoType {
            name: "string".to_string(),
            is_nullable: false,
        };
        assert_eq!(format!("{}", ty), "string");
        assert_eq!(format!("{}", ty2), "string");

        let ty = GoType {
            name: "FooBar".to_string(),
            is_nullable: true,
        };
        assert_eq!(format!("{}", ty), "*FooBar");
        let ty = GoType {
            name: "FooBar".to_string(),
            is_nullable: false,
        };
        assert_eq!(format!("{}", ty), "FooBar");
    }

    #[test]
    fn go_field_display_works() {
        let field = GoField {
            rust_name: "foo_bar".to_string(),
            docs: None,
            ty: GoType {
                name: "string".to_string(),
                is_nullable: true,
            },
        };
        assert_eq!(
            format!("{}", field),
            "FooBar string `json:\"foo_bar,omitempty\"`"
        );

        let field = GoField {
            rust_name: "foo_bar".to_string(),
            docs: None,
            ty: GoType {
                name: "string".to_string(),
                is_nullable: false,
            },
        };
        assert_eq!(format!("{}", field), "FooBar string `json:\"foo_bar\"`");

        let field = GoField {
            rust_name: "foo_bar".to_string(),
            docs: None,
            ty: GoType {
                name: "FooBar".to_string(),
                is_nullable: true,
            },
        };
        assert_eq!(
            format!("{}", field),
            "FooBar *FooBar `json:\"foo_bar,omitempty\"`"
        );
    }

    #[test]
    fn go_field_docs_display_works() {
        let field = GoField {
            rust_name: "foo_bar".to_string(),
            docs: Some("foo_bar is a test field".to_string()),
            ty: GoType {
                name: "string".to_string(),
                is_nullable: true,
            },
        };
        assert_eq!(
            format!("{}", field),
            "// foo_bar is a test field\nFooBar string `json:\"foo_bar,omitempty\"`"
        );
    }

    #[test]
    fn go_type_def_display_works() {
        let ty = GoStruct {
            name: "FooBar".to_string(),
            docs: None,
            fields: vec![GoField {
                rust_name: "foo_bar".to_string(),
                docs: None,
                ty: GoType {
                    name: "string".to_string(),
                    is_nullable: true,
                },
            }],
        };
        assert_eq!(
            format!("{}", ty),
            "type FooBar struct {\n    FooBar string `json:\"foo_bar,omitempty\"`\n}"
        );

        let ty = GoStruct {
            name: "FooBar".to_string(),
            docs: Some("FooBar is a test struct".to_string()),
            fields: vec![GoField {
                rust_name: "foo_bar".to_string(),
                docs: None,
                ty: GoType {
                    name: "string".to_string(),
                    is_nullable: true,
                },
            }],
        };
        assert_eq!(
            format!("{}", ty),
            "// FooBar is a test struct\ntype FooBar struct {\n    FooBar string `json:\"foo_bar,omitempty\"`\n}"
        );
    }
}
