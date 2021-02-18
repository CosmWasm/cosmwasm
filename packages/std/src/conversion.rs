use std::any::type_name;
use std::convert::TryInto;

use crate::errors::{StdError, StdResult};

pub fn to_u32<T: TryInto<u32> + ToString + Copy>(input: T) -> StdResult<u32> {
    input.try_into().map_err(|_| {
        StdError::conversion_err(type_name::<T>(), type_name::<u32>(), input.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_u32_works_for_usize() {
        assert_eq!(to_u32(0usize).unwrap(), 0);
        assert_eq!(to_u32(1usize).unwrap(), 1);
        assert_eq!(to_u32(2147483647usize).unwrap(), 2147483647);
        assert_eq!(to_u32(2147483648usize).unwrap(), 2147483648);
        assert_eq!(to_u32(4294967295usize).unwrap(), 4294967295);

        match to_u32(4294967296usize) {
            Err(StdError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            }) => {
                assert_eq!(from_type, "usize");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "4294967296");
            }
            Err(err) => panic!("unexpected error: {:?}", err),
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
            Err(StdError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            }) => {
                assert_eq!(from_type, "u64");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "4294967296");
            }
            Err(err) => panic!("unexpected error: {:?}", err),
            Ok(_) => panic!("must not succeed"),
        };
    }

    #[test]
    fn to_u32_works_for_i32() {
        assert_eq!(to_u32(0i32).unwrap(), 0);
        assert_eq!(to_u32(1i32).unwrap(), 1);
        assert_eq!(to_u32(2147483647i32).unwrap(), 2147483647);

        match to_u32(-1i32) {
            Err(StdError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            }) => {
                assert_eq!(from_type, "i32");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "-1");
            }
            Err(err) => panic!("unexpected error: {:?}", err),
            Ok(_) => panic!("must not succeed"),
        };
    }
}
