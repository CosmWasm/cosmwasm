/// An iterator that returns all suffixes of a string, excluding the empty string.
///
/// It starts with the full string and ends with the last character.
/// It is a double-ended iterator and can be reversed.
pub fn suffixes(s: &str) -> impl Iterator<Item = &str> + DoubleEndedIterator {
    s.char_indices().map(|(pos, _)| &s[pos..])
}
