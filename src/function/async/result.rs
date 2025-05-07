use crate::Frame;
use crate::ffi::{lua_State, zl_pop};
use crate::state::RawState;
use std::ffi::c_int;

/// Result frame of async call.
pub struct AsyncFrame<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> AsyncFrame<'a, P> {
    #[inline(always)]
    pub(super) fn new(p: &'a mut P) -> Self {
        Self(p)
    }
}

impl<P: Frame> RawState for AsyncFrame<'_, P> {
    #[inline(always)]
    fn state(&mut self) -> *mut lua_State {
        self.0.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.state(), n) };
    }
}
