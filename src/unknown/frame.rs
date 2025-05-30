use crate::Frame;
use crate::ffi::{lua_State, zl_pop};
use crate::state::RawState;
use std::ffi::c_int;

/// Provides [`Frame`] implementation on [Unknown](super::Unknown) value as a workspace.
pub struct UnknownFrame<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> UnknownFrame<'a, P> {
    #[inline(always)]
    pub(super) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }
}

impl<P: Frame> RawState for UnknownFrame<'_, P> {
    #[inline(always)]
    fn state(&mut self) -> *mut lua_State {
        self.0.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.state(), n) };
    }
}
