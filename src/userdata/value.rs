use crate::ffi::{engine_pop, lua_State};
use crate::{Frame, FrameState};
use std::ffi::c_int;

/// Represents a user data in a frame.
pub struct UserValue<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> UserValue<'a, P> {
    /// # Safety
    /// Top of the stack must be a strongly typed user data.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }
}

impl<'a, P: Frame> Drop for UserValue<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}

impl<'a, P: Frame> FrameState for UserValue<'a, P> {
    #[inline(always)]
    fn state(&self) -> *mut lua_State {
        self.0.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { engine_pop(self.state(), n) };
    }
}
