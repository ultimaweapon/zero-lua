use super::UserFrame;
use crate::ffi::zl_pop;
use crate::state::FrameState;
use crate::{Frame, Unknown};
use std::ffi::c_int;
use std::mem::ManuallyDrop;
use std::ops::DerefMut;

/// Represents a user data on the top of stack.
pub struct UserValue<'p, P: Frame>(&'p mut P);

impl<'p, P: Frame> UserValue<'p, P> {
    /// # Safety
    /// Top of the stack must be a strongly typed user data.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'p mut P) -> Self {
        Self(p)
    }

    /// Note that [`Drop`] implementation on [`UserFrame`] will silently fails if `n` is not a valid
    /// index for user value.
    ///
    /// # Panics
    /// If `n` is zero.
    #[inline(always)]
    pub fn set_user_value(&mut self, n: u16) -> UserFrame<Self> {
        unsafe { UserFrame::new(self, n.try_into().unwrap()) }
    }

    #[inline(always)]
    pub fn into_unknown(self) -> Unknown<'p, P> {
        unsafe { Unknown::new(ManuallyDrop::new(self).deref_mut().0) }
    }
}

impl<P: Frame> Drop for UserValue<'_, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}

impl<P: Frame> FrameState for UserValue<'_, P> {
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

impl<'p, P: Frame> From<UserValue<'p, P>> for Unknown<'p, P> {
    #[inline(always)]
    fn from(value: UserValue<'p, P>) -> Self {
        value.into_unknown()
    }
}
