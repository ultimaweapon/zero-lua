use crate::ffi::{lua_State, zl_pop};
use crate::state::RawState;
use crate::{Frame, Unknown};
use std::ffi::c_int;
use std::mem::ManuallyDrop;
use std::ops::DerefMut;

/// Represents a boolean on the top of stack.
pub struct Bool<'p, P: Frame>(&'p mut P);

impl<'p, P: Frame> Bool<'p, P> {
    /// # Safety
    /// Top of the stack must be a boolean.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'p mut P) -> Self {
        Self(p)
    }

    #[inline(always)]
    pub fn into_unknown(self) -> Unknown<'p, P> {
        unsafe { Unknown::new(ManuallyDrop::new(self).deref_mut().0) }
    }
}

impl<P: Frame> Drop for Bool<'_, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}

impl<P: Frame> RawState for Bool<'_, P> {
    #[inline(always)]
    fn state(&mut self) -> *mut lua_State {
        self.0.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.state(), n) };
    }
}

impl<'p, P: Frame> From<Bool<'p, P>> for Unknown<'p, P> {
    #[inline(always)]
    fn from(value: Bool<'p, P>) -> Self {
        value.into_unknown()
    }
}
