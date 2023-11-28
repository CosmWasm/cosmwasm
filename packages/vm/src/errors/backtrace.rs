use core::fmt::{Debug, Display, Formatter, Result};
use std::backtrace::Backtrace;

/// This wraps an actual backtrace to achieve two things:
/// - being able to fill this with a stub implementation in `no_std` environments
/// - being able to use this in conjunction with [`thiserror::Error`]
pub struct BT(Backtrace);

impl BT {
    #[track_caller]
    pub fn capture() -> Self {
        BT(Backtrace::capture())
    }
}

impl Debug for BT {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for BT {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&self.0, f)
    }
}

/// This macro implements `From` for a given error type to a given error type where
/// the target error has a `backtrace` field.
/// This is meant as a replacement for `thiserror`'s `#[from]` attribute, which does not
/// work with our custom backtrace wrapper.
macro_rules! impl_from_err {
    ($from:ty, $to:ty, $map:path) => {
        impl From<$from> for $to {
            fn from(err: $from) -> Self {
                $map {
                    source: err,
                    backtrace: $crate::errors::backtrace::BT::capture(),
                }
            }
        }
    };
}
pub(crate) use impl_from_err;
