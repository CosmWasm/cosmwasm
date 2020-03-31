use std::any::type_name;
use std::convert::TryFrom;

use crate::errors::{ConversionErr, Result};

/// Safely converts input of type FromType to u32.
/// Errors with a cosmwasm_vm::errors::errors if conversion cannot be done.
pub fn to_u32(input: usize) -> Result<u32> {
    u32::try_from(input).or(ConversionErr {
        from_type: type_name::<usize>(),
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
    fn to_u32_works() {
        assert_eq!(to_u32(0).unwrap(), 0);
        assert_eq!(to_u32(1).unwrap(), 1);
        assert_eq!(to_u32(2147483647).unwrap(), 2147483647);
        assert_eq!(to_u32(2147483648).unwrap(), 2147483648);
        assert_eq!(to_u32(4294967295).unwrap(), 4294967295);

        match to_u32(4294967296) {
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
}
