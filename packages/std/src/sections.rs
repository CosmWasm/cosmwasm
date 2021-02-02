/// A sections decoder for the special case of two elements
#[allow(dead_code)] // used in Wasm and tests only
pub fn decode_sections2(data: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let section2_len: usize = if data.len() >= 4 {
        u32::from_be_bytes([
            data[data.len() - 4],
            data[data.len() - 3],
            data[data.len() - 2],
            data[data.len() - 1],
        ]) as usize
    } else {
        panic!("Cannot read section2 length");
    };

    let section1_len_end = data.len() - 4 - section2_len;
    let section1_len: usize = if section1_len_end >= 4 {
        u32::from_be_bytes([
            data[section1_len_end - 4],
            data[section1_len_end - 3],
            data[section1_len_end - 2],
            data[section1_len_end - 1],
        ]) as usize
    } else {
        panic!("Cannot read section1 length");
    };

    if data.len() != 4 + section1_len + 4 + section2_len {
        panic!(
            "Invalid data length: {}, {}, {}",
            data.len(),
            section1_len,
            section2_len
        );
    }

    let mut first = data;
    let mut second = first.split_off(section1_len_end);
    second.truncate(section2_len);
    first.truncate(section1_len);
    (first, second)
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
}
