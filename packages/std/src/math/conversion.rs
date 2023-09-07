/// Grows a big endian signed integer to a bigger size.
/// See <https://en.wikipedia.org/wiki/Sign_extension>
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

/// Shrinks a big endian signed integer to a smaller size.
/// This is the opposite operation of sign extension.
pub fn shrink_be_int<const INPUT_SIZE: usize, const OUTPUT_SIZE: usize>(
    input: [u8; INPUT_SIZE],
) -> Option<[u8; OUTPUT_SIZE]> {
    debug_assert!(INPUT_SIZE >= OUTPUT_SIZE);

    // check bounds
    if input[0] & 0b10000000 != 0 {
        // a negative number should start with only 1s, otherwise it's too small
        for i in &input[0..(INPUT_SIZE - OUTPUT_SIZE)] {
            if *i != 0b11111111u8 {
                return None;
            }
        }
        // the sign bit also has to be 1
        if input[INPUT_SIZE - OUTPUT_SIZE] & 0b10000000 == 0 {
            return None;
        }
    } else {
        // a positive number should start with only 0s, otherwise it's too large
        for i in &input[0..(INPUT_SIZE - OUTPUT_SIZE)] {
            if *i != 0u8 {
                return None;
            }
        }
        // the sign bit also has to be 0
        if input[INPUT_SIZE - OUTPUT_SIZE] & 0b10000000 != 0 {
            return None;
        }
    }

    // Now, we can just copy the last bytes
    let mut output = [0u8; OUTPUT_SIZE];
    output.copy_from_slice(&input[(INPUT_SIZE - OUTPUT_SIZE)..]);
    Some(output)
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

    #[test]
    fn shrink_be_int_works() {
        // test against rust std's integers
        let i32s = [-42, -1, 0i32, 1, 42];
        for i in i32s {
            assert_eq!(
                shrink_be_int(i.to_be_bytes()),
                Some((i as i16).to_be_bytes())
            );
            assert_eq!(
                shrink_be_int(i.to_be_bytes()),
                Some((i as i8).to_be_bytes())
            );
        }
        // these should be too big to fit into an i16 or i8
        let oob = [
            i32::MIN,
            i32::MIN + 10,
            i32::MIN + 1234,
            i32::MAX - 1234,
            i32::MAX - 10,
            i32::MAX,
        ];
        for i in oob {
            // 32 -> 16 bit
            assert_eq!(shrink_be_int::<4, 2>(i.to_be_bytes()), None);
            // 32 -> 8 bit
            assert_eq!(shrink_be_int::<4, 1>(i.to_be_bytes()), None);
        }

        // compare against whole i16 range
        for i in i16::MIN..=i16::MAX {
            let cast = i as i8 as i16;
            if i == cast {
                // if the cast is lossless, `shrink_be_int` should get the same result
                assert_eq!(
                    shrink_be_int::<2, 1>(i.to_be_bytes()),
                    Some((i as i8).to_be_bytes())
                );
            } else {
                // otherwise, we should get None
                assert_eq!(shrink_be_int::<2, 1>(i.to_be_bytes()), None);
            }
        }
    }
}
