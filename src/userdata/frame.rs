use crate::Frame;
use crate::ffi::{lua_State, zl_pop, zl_replace, zl_setiuservalue};
use crate::state::RawState;
use std::ffi::c_int;
use std::num::NonZero;

/// Provides [`Frame`] implementation to set a user value.
pub struct UserFrame<'a, P: Frame> {
    parent: &'a mut P,
    ud: c_int,
    uv: NonZero<u16>,
    has_value: bool,
}

impl<'a, P: Frame> UserFrame<'a, P> {
    /// # Safety
    /// `ud` must be valid.
    #[inline(always)]
    pub(crate) unsafe fn new(parent: &'a mut P, ud: c_int, uv: NonZero<u16>) -> Self {
        Self {
            parent,
            ud,
            uv,
            has_value: false,
        }
    }
}

impl<P: Frame> Drop for UserFrame<'_, P> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.has_value {
            unsafe { zl_setiuservalue(self.parent.state(), self.ud, self.uv.get()) };
        }
    }
}

impl<P: Frame> RawState for UserFrame<'_, P> {
    #[inline(always)]
    fn state(&mut self) -> *mut lua_State {
        self.parent.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        // Remove excess values.
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
