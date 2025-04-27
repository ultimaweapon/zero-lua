use crate::ffi::{zl_pop, zl_replace, zl_setiuservalue};
use crate::{Frame, FrameState};
use std::ffi::c_int;
use std::num::NonZero;

/// Provides [`Frame`] implementation to set a user value.
pub struct UserFrame<'a, P: Frame> {
    parent: &'a mut P,
    uv: NonZero<u16>,
    has_value: bool,
}

impl<'a, P: Frame> UserFrame<'a, P> {
    /// # Safety
    /// Top of the stack must be a full userdata.
    #[inline(always)]
    pub(crate) unsafe fn new(parent: &'a mut P, uv: NonZero<u16>) -> Self {
        Self {
            parent,
            uv,
            has_value: false,
        }
    }
}

impl<'a, P: Frame> Drop for UserFrame<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.has_value {
            unsafe { zl_setiuservalue(self.parent.state().get(), -2, self.uv.get()) };
        }
    }
}

impl<'a, P: Frame> FrameState for UserFrame<'a, P> {
    type State = P::State;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        self.parent.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        // Remove excess values.
        let excess = n - 1;

        if excess > 0 {
            unsafe { zl_pop(self.state().get(), excess) };
        }

        if self.has_value {
            unsafe { zl_replace(self.state().get(), -2) };
        }

        self.has_value = true;
    }
}
