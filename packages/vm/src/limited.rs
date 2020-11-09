//! A set of tools designed for processing user defined contract data,
//! which can potientially have abusive size.

use std::collections::{BTreeSet, HashSet};
use std::iter::FromIterator;

pub trait LimitedDisplay {
    /// Returns a string representationof the object, which is shorter than or equal to `max_length`.
    /// Implementations must panic if `max_length` is not reasonably large.
    fn to_string_limited(&self, max_length: usize) -> String;
}

impl<E: Ord + AsRef<str>> LimitedDisplay for HashSet<E> {
    fn to_string_limited(&self, max_length: usize) -> String {
        let sorted = BTreeSet::from_iter(self);
        let mut out = String::with_capacity(max_length * 130 / 100);
        let opening = "{";
        let closing = "}";

        let mut first = true;
        out.push_str(opening);
        let mut lengths_stack = Vec::<usize>::new();
        for element in sorted.iter() {
            lengths_stack.push(out.len());

            if first {
                out.push('"');
                first = false;
            } else {
                out.push_str(", \"");
            }
            out.push_str(element.as_ref());
            out.push('"');

            if out.len() > max_length {
                break;
            };
        }

        if out.len() + closing.len() <= max_length {
            out.push_str(closing);
            out
        } else {
            loop {
                let previous_length = lengths_stack
                    .pop()
                    .expect("Cannot remove hide enough elements to fit in length limit.");
                let skipped = sorted.len() - lengths_stack.len();
                let remaining = sorted.len() - skipped;
                let skipped_text = if remaining == 0 {
                    format!("... {} elements", skipped)
                } else {
                    format!(", ... {} more", skipped)
                };
                if previous_length + skipped_text.len() + closing.len() <= max_length {
                    out.truncate(previous_length);
                    out.push_str(&skipped_text);
                    out.push_str(closing);
                    return out;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn works_for_hashset() {
        let set = HashSet::<String>::new();
        assert_eq!(set.to_string_limited(100), "{}");
        assert_eq!(set.to_string_limited(20), "{}");
        assert_eq!(set.to_string_limited(2), "{}");

        let fruits = HashSet::from_iter(
            [
                "watermelon".to_string(),
                "apple".to_string(),
                "banana".to_string(),
            ]
            .iter()
            .cloned(),
        );
        assert_eq!(
            fruits.to_string_limited(100),
            "{\"apple\", \"banana\", \"watermelon\"}"
        );
        assert_eq!(
            fruits.to_string_limited(33),
            "{\"apple\", \"banana\", \"watermelon\"}"
        );
        assert_eq!(
            fruits.to_string_limited(32),
            "{\"apple\", \"banana\", ... 1 more}"
        );
        assert_eq!(
            fruits.to_string_limited(31),
            "{\"apple\", \"banana\", ... 1 more}"
        );
        assert_eq!(fruits.to_string_limited(30), "{\"apple\", ... 2 more}");
        assert_eq!(fruits.to_string_limited(21), "{\"apple\", ... 2 more}");
        assert_eq!(fruits.to_string_limited(20), "{... 3 elements}");
        assert_eq!(fruits.to_string_limited(16), "{... 3 elements}");
    }

    #[test]
    #[should_panic(expected = "Cannot remove hide enough elements to fit in length limit.")]
    fn panics_if_limit_is_too_small_empty() {
        let set = HashSet::<String>::new();
        assert_eq!(set.to_string_limited(1), "{}");
    }

    #[test]
    #[should_panic(expected = "Cannot remove hide enough elements to fit in length limit.")]
    fn panics_if_limit_is_too_small_nonempty() {
        let fruits = HashSet::from_iter(
            [
                "watermelon".to_string(),
                "apple".to_string(),
                "banana".to_string(),
            ]
            .iter()
            .cloned(),
        );
        assert_eq!(fruits.to_string_limited(15), "{... 3 elements}");
    }
}
