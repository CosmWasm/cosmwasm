/// A sections decoder for the special case of two elements
#[allow(dead_code)] // used in Wasm and tests only
pub fn decode_sections2(data: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let (rest, second) = split_tail(data);
    let (_, first) = split_tail(rest);
    (first, second)
}

/// Splits data into the last section ("tail") and the rest.
/// The tail's length information is cut off, such that it is ready to use.
/// The rest is basically unparsed and contails the lengths of the remaining sections.
///
/// While the tail is copied into a new vector, the rest is only truncated such that
/// no re-allocation is necessary.
///
/// If `data` contains one section only, `data` is moved into the tail entirely
fn split_tail(data: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let tail_len: usize = if data.len() >= 4 {
        u32::from_be_bytes([
            data[data.len() - 4],
            data[data.len() - 3],
            data[data.len() - 2],
            data[data.len() - 1],
        ]) as usize
    } else {
        panic!("Cannot read section length");
    };
    let rest_len_end = data.len() - 4 - tail_len;

    let (rest, mut tail) = if rest_len_end == 0 {
        // i.e. all data is the tail
        (Vec::new(), data)
    } else {
        let mut rest = data;
        let tail = rest.split_off(rest_len_end);
        (rest, tail)
    };
    tail.truncate(tail_len); // cut off length
    (rest, tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_sections2_works() {
        let data = b"\xAA\0\0\0\x01\xBB\xCC\0\0\0\x02".to_vec();
        assert_eq!(decode_sections2(data), (vec![0xAA], vec![0xBB, 0xCC]));

        let data = b"\xDE\xEF\x62\0\0\0\x03\0\0\0\0".to_vec();
        assert_eq!(decode_sections2(data), (vec![0xDE, 0xEF, 0x62], vec![]));

        let data = b"\0\0\0\0\xDE\xEF\x62\0\0\0\x03".to_vec();
        assert_eq!(decode_sections2(data), (vec![], vec![0xDE, 0xEF, 0x62]));

        let data = b"\0\0\0\0\0\0\0\0".to_vec();
        assert_eq!(decode_sections2(data), (vec![], vec![]));

        let data = b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\x13\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\x9D\0\0\x01\x15".to_vec();
        assert_eq!(decode_sections2(data), (vec![0xFF; 19], vec![0x9D; 277]));
    }

    #[test]
    fn decode_sections2_preserved_first_vector() {
        let original = b"\xAA\0\0\0\x01\xBB\xCC\0\0\0\x02".to_vec();
        let original_capacity = original.capacity();
        let original_ptr = original.as_ptr();
        let (first, second) = decode_sections2(original);

        // This is not copied
        assert_eq!(first.capacity(), original_capacity);
        assert_eq!(first.as_ptr(), original_ptr);

        // This is a copy
        assert_ne!(second.capacity(), original_capacity);
        assert_ne!(second.as_ptr(), original_ptr);
    }
}
