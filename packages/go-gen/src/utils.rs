/// An iterator that returns all suffixes of a string, excluding the empty string.
///
/// It starts with the full string and ends with the last character.
/// It is a double-ended iterator and can be reversed.
pub fn suffixes(s: &str) -> impl Iterator<Item = &str> + DoubleEndedIterator {
    s.char_indices().map(|(pos, _)| &s[pos..])
}

/// Replaces common pascal-case acronyms with their uppercase counterparts.
pub fn replace_acronyms(ty: impl Into<String>) -> String {
    let mut ty = ty.into();
    replace_word_in_place(&mut ty, "Url", "URL");
    replace_word_in_place(&mut ty, "Uri", "URI");
    replace_word_in_place(&mut ty, "Id", "ID");
    replace_word_in_place(&mut ty, "Ibc", "IBC");
    ty
}

fn replace_word_in_place(haystack: &mut String, from: &str, to: &str) {
    assert_eq!(from.len(), to.len(), "from and to must be the same length");
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(from) {
        let begin = start + pos;
        let end = start + pos + from.len();
        let next_char = haystack.chars().nth(end);
        match next_char {
            Some(next_char) if next_char.is_ascii_lowercase() => {}
            _ => {
                // if the next character is uppercase or any non-ascii char or
                // there is no next char, it's a full word
                haystack.replace_range(begin..end, to);
            }
        }
        start = end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic = "from and to must be the same length"]
    fn replace_in_place_different_lengths() {
        let mut s = "foo".to_string();
        replace_word_in_place(&mut s, "foo", "barbar");
    }

    #[test]
    fn replace_in_place_multiple_works() {
        let mut s = "FooFooFooFooFoo".to_string();
        replace_word_in_place(&mut s, "Foo", "bar");
        assert_eq!(s, "barbarbarbarbar");
    }

    #[test]
    fn replace_in_place_single_works() {
        let mut s = "foo".to_string();
        replace_word_in_place(&mut s, "foo", "bar");
        assert_eq!(s, "bar");
    }

    #[test]
    fn replace_word_in_place_part() {
        let mut s = "Foofoo".to_string();
        replace_word_in_place(&mut s, "Foo", "Bar");
        // should not replace, because it's not a full word
        assert_eq!(s, "Foofoo");
    }

    #[test]
    fn replace_acronyms_works() {
        assert_eq!(replace_acronyms("MyIdentity"), "MyIdentity");
        assert_eq!(replace_acronyms("MyIdentityId"), "MyIdentityID");
        assert_eq!(replace_acronyms("MyUri"), "MyURI");
        assert_eq!(replace_acronyms("Url"), "URL");
        assert_eq!(replace_acronyms("A"), "A");
        assert_eq!(replace_acronyms("Url🦦"), "URL🦦");
    }
}
