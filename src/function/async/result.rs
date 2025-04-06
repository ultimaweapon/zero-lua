use crate::ffi::engine_pop;
use crate::{Frame, FrameState};
use std::ffi::c_int;

/// Result frame of async call.
pub struct AsyncFrame<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> AsyncFrame<'a, P> {
    #[inline(always)]
    pub(super) fn new(p: &'a mut P) -> Self {
        Self(p)
    }
}

impl<'a, P: Frame> FrameState for AsyncFrame<'a, P> {
    type State = P::State;

    #[inline(always)]
    fn state(&self) -> &Self::State {
        self.0.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { engine_pop(self.state().get(), n) };
    }
}
