use crate::Frame;

/// Encapsulates Lua nil value in a frame.
pub struct Nil<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> Nil<'a, P> {
    /// # Safety
    /// Top of the stack must be nil value.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }
}

impl<'a, P: Frame> Drop for Nil<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}
