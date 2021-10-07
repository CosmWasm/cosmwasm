//! A module containing an assertion framework for CosmWasm contracts.
//! The methods in here never panic but return errors instead.

/// Quick check for a guard. If the condition (first argument) is false,
/// then return the second argument wrapped in Err(x).
///
///   ensure!(permissions.delegate, ContractError::DelegatePerm {});
///  is the same as
///   if !permissions.delegate {
///     return Err(ContractError::DelegatePerm {});
///   }
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            return Err(std::convert::From::from($e));
        }
    };
}

/// Quick check for a guard. Like assert_eq!, but rather than panic,
/// it returns the second argument wrapped in Err(x).
///
///   ensure_eq!(info.sender, cfg.admin, ContractError::Unauthorized {});
///  is the same as
///   if info.sender != cfg.admin {
///     return Err(ContractError::Unauthorized {});
///   }
#[macro_export]
macro_rules! ensure_eq {
    ($a:expr, $b:expr, $e:expr) => {
        ensure!($a == $b, $e);
    };
}

#[cfg(test)]
mod tests {
    use crate::StdError;

    #[test]
    fn ensure_works() {
        fn check(a: usize, b: usize) -> Result<(), StdError> {
            ensure!(a == b, StdError::generic_err("foobar"));
            Ok(())
        }

        let err = check(5, 6).unwrap_err();
        assert!(matches!(err, StdError::GenericErr { .. }));

        check(5, 5).unwrap();
    }

    #[test]
    fn ensure_can_infer_error_type() {
        let check = |a, b| {
            ensure!(a == b, StdError::generic_err("foobar"));
            Ok(())
        };

        let err = check(5, 6).unwrap_err();
        assert!(matches!(err, StdError::GenericErr { .. }));

        check(5, 5).unwrap();
    }

    #[test]
    fn ensure_can_convert_into() {
        #[derive(Debug)]
        struct ContractError;

        impl From<StdError> for ContractError {
            fn from(_original: StdError) -> Self {
                ContractError
            }
        }

        fn check(a: usize, b: usize) -> Result<(), ContractError> {
            ensure!(a == b, StdError::generic_err("foobar"));
            Ok(())
        }

        let err = check(5, 6).unwrap_err();
        assert!(matches!(err, ContractError));

        check(5, 5).unwrap();
    }

    #[test]
    fn ensure_eq_works() {
        let check = |a, b| {
            ensure_eq!(a, b, StdError::generic_err("foobar"));
            Ok(())
        };

        let err = check("123", "456").unwrap_err();
        assert!(matches!(err, StdError::GenericErr { .. }));

        check("123", "123").unwrap();
    }
}
