pub use self::frame::*;
pub use self::setter::*;

use crate::ffi::{zl_gettop, zl_pop};
use crate::state::FrameState;
use crate::{Frame, PositiveInt};
use std::ffi::c_int;

mod frame;
mod setter;

/// Represents an unknown value on the top of stack.
pub struct Unknown<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> Unknown<'a, P> {
    /// # Safety
    /// Ownership of the top stack will be transferred to the returned [`Unknown`].
    #[inline(always)]
    pub(crate) unsafe fn new(p: *mut P) -> Self {
        Self(unsafe { &mut *p })
    }

    #[inline(always)]
    pub fn set(&mut self) -> (UnknownSetter, UnknownFrame<Self>) {
        let state = self.0.state().get();
        let index = unsafe { zl_gettop(state) };
        let index = unsafe { PositiveInt::new_unchecked(index) };

        unsafe { (UnknownSetter::new(state, index), UnknownFrame::new(self)) }
    }
}

impl<'a, P: Frame> Drop for Unknown<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}

impl<'a, P: Frame> FrameState for Unknown<'a, P> {
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
