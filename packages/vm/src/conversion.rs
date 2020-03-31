use std::any::type_name;
use std::convert::TryInto;
use std::fmt::Display;

use crate::errors::{ConversionErr, Result};

/// Safely converts input of type T to u32.
/// Errors with a cosmwasm_vm::errors::Error::ConversionErr if conversion cannot be done.
pub fn to_u32<T: TryInto<u32> + Display + Copy>(input: T) -> Result<u32> {
    input.try_into().or(ConversionErr {
        from_type: type_name::<T>(),
        to_type: type_name::<u32>(),
        input: format!("{}", input),
    }
    .fail())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::Error;

    #[test]
    fn to_u32_works_for_usize() {
        assert_eq!(to_u32(0usize).unwrap(), 0);
        assert_eq!(to_u32(1usize).unwrap(), 1);
        assert_eq!(to_u32(2147483647usize).unwrap(), 2147483647);
        assert_eq!(to_u32(2147483648usize).unwrap(), 2147483648);
        assert_eq!(to_u32(4294967295usize).unwrap(), 4294967295);

        match to_u32(4294967296usize) {
            Err(Error::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            }) => {
                assert_eq!(from_type, "usize");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "4294967296");
            }
            Err(err) => panic!("unexpected error: {:}", err),
            Ok(_) => panic!("must not succeed"),
        };
    }

    #[test]
    fn to_u32_works_for_u64() {
        assert_eq!(to_u32(0u64).unwrap(), 0);
        assert_eq!(to_u32(1u64).unwrap(), 1);
        assert_eq!(to_u32(2147483647u64).unwrap(), 2147483647);
        assert_eq!(to_u32(2147483648u64).unwrap(), 2147483648);
        assert_eq!(to_u32(4294967295u64).unwrap(), 4294967295);

        match to_u32(4294967296u64) {
            Err(Error::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            }) => {
                assert_eq!(from_type, "u64");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "4294967296");
            }
            Err(err) => panic!("unexpected error: {:}", err),
            Ok(_) => panic!("must not succeed"),
        };
    }

    #[test]
    fn to_u32_works_for_i32() {
        assert_eq!(to_u32(0i32).unwrap(), 0);
        assert_eq!(to_u32(1i32).unwrap(), 1);
        assert_eq!(to_u32(2147483647i32).unwrap(), 2147483647);

        match to_u32(-1i32) {
            Err(Error::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            }) => {
                assert_eq!(from_type, "i32");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "-1");
            }
            Err(err) => panic!("unexpected error: {:}", err),
            Ok(_) => panic!("must not succeed"),
        };
    }
}
