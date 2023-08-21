/// An iterator that returns all suffixes of a string, excluding the empty string.
///
/// It starts with the full string and ends with the last character.
/// It is a double-ended iterator and can be reversed.
pub fn suffixes(s: &str) -> impl Iterator<Item = &str> + DoubleEndedIterator {
    s.char_indices().map(|(pos, _)| &s[pos..])
}

/// Replaces common pascal-case acronyms with their uppercase counterparts.
pub fn replace_acronyms(ty: &str) -> String {
    let mut ty = ty.replace("Url", "URL");
    replace_in_place(&mut ty, "Uri", "URI");
    replace_in_place(&mut ty, "Id", "ID");
    replace_in_place(&mut ty, "Ibc", "IBC");
    ty
}

pub fn replace_in_place(haystack: &mut String, from: &str, to: &str) {
    assert_eq!(from.len(), to.len(), "from and to must be the same length");
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(from) {
        let begin = start + pos;
        let end = start + pos + from.len();
        haystack.replace_range(begin..end, to);
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
        replace_in_place(&mut s, "foo", "barbar");
    }

    #[test]
    fn replace_in_place_multiple_works() {
        let mut s = "foofoofoofoofoo".to_string();
        replace_in_place(&mut s, "foo", "bar");
        assert_eq!(s, "barbarbarbarbar");
    }

    #[test]
    fn replace_in_place_single_works() {
        let mut s = "foo".to_string();
        replace_in_place(&mut s, "foo", "bar");
        assert_eq!(s, "bar");
    }
}
