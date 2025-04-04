pub use self::ret::*;

use crate::ffi::{engine_checkstack, zl_pcall};
use crate::{Frame, Str};

mod ret;

/// Encapsulates a callable object on the top of Lua stack.
pub struct Function<'a, P: Frame>(Option<&'a mut P>);

impl<'a, P: Frame> Function<'a, P> {
    /// # Safety
    /// Top of the stack must be a callable object.
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(Some(p))
    }

    pub fn call<R: FuncRet<'a, P>>(mut self) -> Result<R, Str<'a, P>> {
        // Ensure stack for results. We can't take out the parent here since engine_checkstack can
        // throw a C++ exception.
        if R::N > 0 {
            unsafe { engine_checkstack(self.0.as_ref().unwrap().state().get(), R::N) };
        }

        // Call.
        let p = self.0.take().unwrap();

        match unsafe { zl_pcall(p.state().get(), 0, R::N, 0) } {
            true => Ok(unsafe { R::new(p) }),
            false => Err(unsafe { Str::new(p) }),
        }
    }
}

impl<'a, P: Frame> Drop for Function<'a, P> {
    fn drop(&mut self) {
        if let Some(p) = self.0.take() {
            unsafe { p.release_values(1) };
        }
    }
}
