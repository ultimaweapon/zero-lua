use crate::{Frame, FrameValue};
use std::num::NonZero;

/// Represents a boolean on the top of stack.
pub struct Bool<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> Bool<'a, P> {
    /// # Safety
    /// Top of the stack must be a boolean.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }
}

impl<P: Frame> Drop for Bool<'_, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(Self::N.get().into()) };
    }
}

unsafe impl<'a, P: Frame> FrameValue<'a, P> for Bool<'a, P> {
    const N: NonZero<u8> = NonZero::new(1).unwrap();
}
