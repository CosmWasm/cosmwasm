use crate::conversion::to_u32;
use crate::errors::VmResult;

/// Encodes multiple sections of data into one vector.
///
/// Each section is suffixed by a section length encoded as big endian uint32.
/// Using suffixes instead of prefixes allows reading sections in reverse order,
/// such that the first element does not need to be re-allocated if the contract's
/// data structure supports truncation (such as a Rust vector).
///
/// The resulting data looks like this:
///
/// ```ignore
/// section1 || section1_len || section2 || section2_len || section3 || section3_len || …
/// ```
#[allow(dead_code)]
pub fn encode_sections(sections: &[Vec<u8>]) -> VmResult<Vec<u8>> {
    let mut out_len: usize = sections.iter().map(|section| section.len()).sum();
    out_len += 4 * sections.len();
    let mut out_data = Vec::with_capacity(out_len);
    for section in sections {
        let section_len = to_u32(section.len())?.to_be_bytes();
        out_data.extend(section);
        out_data.extend_from_slice(&section_len);
    }
    debug_assert_eq!(out_data.len(), out_len);
    debug_assert_eq!(out_data.capacity(), out_len);
    Ok(out_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_sections_works_for_empty_sections() {
        let enc = encode_sections(&[]).unwrap();
        assert_eq!(enc, b"" as &[u8]);
        let enc = encode_sections(&[vec![]]).unwrap();
        assert_eq!(enc, b"\0\0\0\0" as &[u8]);
        let enc = encode_sections(&[vec![], vec![]]).unwrap();
        assert_eq!(enc, b"\0\0\0\0\0\0\0\0" as &[u8]);
        let enc = encode_sections(&[vec![], vec![], vec![]]).unwrap();
        assert_eq!(enc, b"\0\0\0\0\0\0\0\0\0\0\0\0" as &[u8]);
    }

    #[test]
    fn encode_sections_works_for_one_element() {
        let enc = encode_sections(&[]).unwrap();
        assert_eq!(enc, b"" as &[u8]);
        let enc = encode_sections(&[vec![0xAA]]).unwrap();
        assert_eq!(enc, b"\xAA\0\0\0\x01" as &[u8]);
        let enc = encode_sections(&[vec![0xAA, 0xBB]]).unwrap();
        assert_eq!(enc, b"\xAA\xBB\0\0\0\x02" as &[u8]);
        let enc = encode_sections(&[vec![0x9D; 277]]).unwrap();
        assert_eq!(enc, b"\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\0\0\x01\x15" as &[u8]);
    }

    #[test]
    fn encode_sections_works_for_multiple_elements() {
        let enc = encode_sections(&[vec![0xAA]]).unwrap();
        assert_eq!(enc, b"\xAA\0\0\0\x01" as &[u8]);
        let enc = encode_sections(&[vec![0xAA], vec![0xDE, 0xDE]]).unwrap();
        assert_eq!(enc, b"\xAA\0\0\0\x01\xDE\xDE\0\0\0\x02" as &[u8]);
        let enc = encode_sections(&[vec![0xAA], vec![0xDE, 0xDE], vec![]]).unwrap();
        assert_eq!(enc, b"\xAA\0\0\0\x01\xDE\xDE\0\0\0\x02\0\0\0\0" as &[u8]);
        let enc = encode_sections(&[vec![0xAA], vec![0xDE, 0xDE], vec![], vec![0xFF; 19]]).unwrap();
        assert_eq!(enc, b"\xAA\0\0\0\x01\xDE\xDE\0\0\0\x02\0\0\0\0\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\x13" as &[u8]);
    }
}