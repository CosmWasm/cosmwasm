use std::fmt::Display;

use super::std_error::{DynContractErr, InvalidBase64, StdError, Unauthorized, Underflow};

pub fn dyn_contract_err<S: Into<String>>(msg: S) -> StdError {
    DynContractErr { msg: msg.into() }.build()
}

pub fn invalid_base64<S: Display>(msg: S) -> StdError {
    InvalidBase64 {
        msg: msg.to_string(),
    }
    .build()
}

pub fn underflow<U: ToString>(minuend: U, subtrahend: U) -> StdError {
    Underflow {
        minuend: minuend.to_string(),
        subtrahend: subtrahend.to_string(),
    }
    .build()
}

pub fn unauthorized() -> StdError {
    Unauthorized {}.build()
}

#[cfg(test)]
mod test {
    use super::super::std_error::StdError;
    use super::*;

    // example of reporting contract errors with format!
    #[test]
    fn dyn_contract_err_owned() {
        let guess = 7;
        let error: StdError = dyn_contract_err(format!("{} is too low", guess));
        match error {
            StdError::DynContractErr { msg, .. } => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            e => panic!("unexpected error, {:?}", e),
        }
    }

    // example of reporting static contract errors
    #[test]
    fn dyn_contract_err_ref() {
        let error: StdError = dyn_contract_err("not implemented");
        match error {
            StdError::DynContractErr { msg, .. } => assert_eq!(msg, "not implemented"),
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
