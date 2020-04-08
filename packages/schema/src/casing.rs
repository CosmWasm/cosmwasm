pub fn to_snake_case(name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in name.char_indices() {
        if index != 0 && ch.is_uppercase() {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_snake_case_leaves_snake_case_untouched() {
        assert_eq!(to_snake_case(""), "");
        assert_eq!(to_snake_case("a"), "a");
        assert_eq!(to_snake_case("abc"), "abc");
        assert_eq!(to_snake_case("a_bc"), "a_bc");
    }

    #[test]
    fn to_snake_case_works_for_camel_case() {
        assert_eq!(to_snake_case("Foobar"), "foobar");
        assert_eq!(to_snake_case("FooBar"), "foo_bar");
        assert_eq!(to_snake_case("ABC"), "a_b_c");
    }
}
