use alloc::boxed::Box;
use core::fmt::{Debug, Display, Formatter, Result};

/// This wraps an actual backtrace to achieve two things:
/// - being able to fill this with a stub implementation in `no_std` environments
/// - being able to use this in conjunction with [`thiserror::Error`]
pub struct BT(Box<dyn Printable + Sync + Send>);

impl BT {
    #[track_caller]
    pub fn capture() -> Self {
        // in case of no_std, we can fill with a stub here
        #[cfg(feature = "std")]
        {
            #[cfg(target_arch = "wasm32")]
            return BT(Box::new(std::backtrace::Backtrace::disabled()));
            #[cfg(not(target_arch = "wasm32"))]
            return BT(Box::new(std::backtrace::Backtrace::capture()));
        }
        #[cfg(not(feature = "std"))]
        {
            BT(Box::new(Stub))
        }
    }
}

trait Printable: Debug + Display {}
impl<T> Printable for T where T: Debug + Display {}

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

#[allow(unused)]
struct Stub;

impl Debug for Stub {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "<disabled>")
    }
}

impl Display for Stub {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "<disabled>")
    }
}
