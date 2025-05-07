use crate::ffi::zl_pop;
use crate::state::RawState;
use crate::{Frame, Unknown};
use std::ffi::c_int;
use std::mem::ManuallyDrop;
use std::ops::DerefMut;

/// Encapsulates Lua nil value in a frame.
pub struct Nil<'p, P: Frame>(&'p mut P);

impl<'p, P: Frame> Nil<'p, P> {
    /// # Safety
    /// Top of the stack must be nil value.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'p mut P) -> Self {
        Self(p)
    }

    #[inline(always)]
    pub fn into_unknown(self) -> Unknown<'p, P> {
        unsafe { Unknown::new(ManuallyDrop::new(self).deref_mut().0) }
    }
}

impl<P: Frame> Drop for Nil<'_, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}

impl<P: Frame> RawState for Nil<'_, P> {
    type State = P::State;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        self.0.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.state().get(), n) };
    }
}

impl<'p, P: Frame> From<Nil<'p, P>> for Unknown<'p, P> {
    #[inline(always)]
    fn from(value: Nil<'p, P>) -> Self {
        value.into_unknown()
    }
}
