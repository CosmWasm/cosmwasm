/// Grows a big endian signed integer to a bigger size.
pub const fn grow_be_int<const INPUT_SIZE: usize, const OUTPUT_SIZE: usize>(
    input: [u8; INPUT_SIZE],
) -> [u8; OUTPUT_SIZE] {
    debug_assert!(INPUT_SIZE <= OUTPUT_SIZE);
    // check if sign bit is set
    let mut output = if input[0] & 0b10000000 != 0 {
        // negative number is filled up with 1s
        [0b11111111u8; OUTPUT_SIZE]
    } else {
        [0u8; OUTPUT_SIZE]
    };
    let mut i = 0;

    // copy input to the end of output
    // copy_from_slice is not const, so we have to do this manually
    while i < INPUT_SIZE {
        output[OUTPUT_SIZE - INPUT_SIZE + i] = input[i];
        i += 1;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grow_be_int_works() {
        // test against rust std's integers
        let i32s = [i32::MIN, -1, 0, 1, 42, i32::MAX];
        for i in i32s {
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i64).to_be_bytes());
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i128).to_be_bytes());
        }
        let i8s = [i8::MIN, -1, 0, 1, 42, i8::MAX];
        for i in i8s {
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i16).to_be_bytes());
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i32).to_be_bytes());
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i64).to_be_bytes());
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i128).to_be_bytes());
        }
    }
}
