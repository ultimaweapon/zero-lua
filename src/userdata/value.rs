use super::UserFrame;
use crate::ffi::zl_pop;
use crate::{Frame, FrameState, FrameValue};
use std::ffi::c_int;
use std::num::NonZero;

/// Represents a user data on the top of stack.
pub struct UserValue<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> UserValue<'a, P> {
    /// # Safety
    /// Top of the stack must be a strongly typed user data.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
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
}

impl<P: Frame> Drop for UserValue<'_, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(Self::N.get().into()) };
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

unsafe impl<'a, P: Frame> FrameValue<'a, P> for UserValue<'a, P> {
    const N: NonZero<u8> = NonZero::new(1).unwrap();
}
