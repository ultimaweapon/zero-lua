use crate::Frame;
use crate::ffi::{lua_State, zl_pop, zl_replace, zl_setglobal};
use crate::state::RawState;
use std::ffi::{CStr, c_int};

/// Provides [`Frame`] implementation to set a global value.
///
/// Only the first value is used if you push a multiple values (the rest will be discarded) and only
/// the value from the recent push will be used.
pub struct GlobalSetter<'a, P, N>
where
    P: Frame,
    N: AsRef<CStr>,
{
    parent: &'a mut P,
    name: N,
    has_value: bool,
}

impl<'a, P, N> GlobalSetter<'a, P, N>
where
    P: Frame,
    N: AsRef<CStr>,
{
    #[inline(always)]
    pub(crate) fn new(parent: &'a mut P, name: N) -> Self {
        Self {
            parent,
            name,
            has_value: false,
        }
    }
}

impl<P, N> Drop for GlobalSetter<'_, P, N>
where
    P: Frame,
    N: AsRef<CStr>,
{
    #[inline(always)]
    fn drop(&mut self) {
        if self.has_value {
            unsafe { zl_setglobal(self.state(), self.name.as_ref().as_ptr()) };
        }
    }
}

impl<P, N> RawState for GlobalSetter<'_, P, N>
where
    P: Frame,
    N: AsRef<CStr>,
{
    #[inline(always)]
    fn state(&mut self) -> *mut lua_State {
        self.parent.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        // All checks here should be optimized away in most cases since this method and new() should
        // be automatically inlined.
        let excess = n - 1;

        if excess > 0 {
            unsafe { zl_pop(self.state(), excess) };
        }

        if self.has_value {
            unsafe { zl_replace(self.state(), -2) };
        }

        self.has_value = true;
    }
}
