use std::fmt::Display;

use super::std_error::StdError;

#[deprecated(note = "Please use StdError::generic_err function instead")]
pub fn generic_err<S: Into<String>>(msg: S) -> StdError {
    StdError::generic_err(msg)
}

#[deprecated(note = "Please use StdError::invalid_base64 function instead")]
pub fn invalid_base64<S: Display>(msg: S) -> StdError {
    StdError::invalid_base64(msg)
}

#[deprecated(note = "Please use StdError::invalid_utf8 function instead")]
pub fn invalid_utf8<S: Display>(msg: S) -> StdError {
    StdError::invalid_utf8(msg)
}

#[deprecated(note = "Please use StdError::not_found function instead")]
pub fn not_found<S: Into<String>>(kind: S) -> StdError {
    StdError::not_found(kind)
}

#[deprecated(note = "Please use StdError::parse_err function instead")]
pub fn parse_err<T: Into<String>, M: Display>(target: T, msg: M) -> StdError {
    StdError::parse_err(target, msg)
}

#[deprecated(note = "Please use StdError::serialize_err function instead")]
pub fn serialize_err<S: Into<String>, M: Display>(source: S, msg: M) -> StdError {
    StdError::serialize_err(source, msg)
}

#[deprecated(note = "Please use StdError::underflow function instead")]
pub fn underflow<U: ToString>(minuend: U, subtrahend: U) -> StdError {
    StdError::underflow(minuend, subtrahend)
}

#[deprecated(note = "Please use StdError::unauthorized function instead")]
pub fn unauthorized() -> StdError {
    StdError::unauthorized()
}

#[allow(deprecated)]
#[cfg(test)]
mod test {
    use super::super::std_error::StdError;
    use super::*;

    // example of reporting contract errors with format!
    #[test]
    fn generic_err_owned() {
        let guess = 7;
        let error: StdError = generic_err(format!("{} is too low", guess));
        match error {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            e => panic!("unexpected error, {:?}", e),
        }
    }

    // example of reporting static contract errors
    #[test]
    fn generic_err_ref() {
        let error: StdError = generic_err("not implemented");
        match error {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "not implemented"),
            e => panic!("unexpected error, {:?}", e),
        }
    }

    #[test]
    fn invalid_base64_works_for_strings() {
        let error: StdError = invalid_base64("my text");
        match error {
            StdError::InvalidBase64 { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_base64_works_for_errors() {
        let original = base64::DecodeError::InvalidLength;
        let error: StdError = invalid_base64(original);
        match error {
            StdError::InvalidBase64 { msg, .. } => {
                assert_eq!(msg, "Encoded text cannot have a 6-bit remainder.");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_utf8_works_for_strings() {
        let error: StdError = invalid_utf8("my text");
        match error {
            StdError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_utf8_works_for_errors() {
        let original = String::from_utf8(vec![0x80]).unwrap_err();
        let error: StdError = invalid_utf8(original);
        match error {
            StdError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 1 bytes from index 0");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn not_found_works() {
        let error: StdError = not_found("gold");
        match error {
            StdError::NotFound { kind, .. } => assert_eq!(kind, "gold"),
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn parse_err_works() {
        let error: StdError = parse_err("Book", "Missing field: title");
        match error {
            StdError::ParseErr { target, msg, .. } => {
                assert_eq!(target, "Book");
                assert_eq!(msg, "Missing field: title");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn serialize_err_works() {
        let error: StdError = serialize_err("Book", "Content too long");
        match error {
            StdError::SerializeErr { source, msg, .. } => {
                assert_eq!(source, "Book");
                assert_eq!(msg, "Content too long");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn underflow_works_for_u128() {
        let error: StdError = underflow(123u128, 456u128);
        match error {
            StdError::Underflow {
                minuend,
                subtrahend,
                ..
            } => {
                assert_eq!(minuend, "123");
                assert_eq!(subtrahend, "456");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn underflow_works_for_i64() {
        let error: StdError = underflow(777i64, 1234i64);
        match error {
            StdError::Underflow {
                minuend,
                subtrahend,
                ..
            } => {
                assert_eq!(minuend, "777");
                assert_eq!(subtrahend, "1234");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn unauthorized_works() {
        let error: StdError = unauthorized();
        match error {
            StdError::Unauthorized { .. } => {}
            _ => panic!("expect different error"),
        }
    }
}
