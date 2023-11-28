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
