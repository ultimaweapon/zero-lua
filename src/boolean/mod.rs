use crate::Frame;

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

impl<'a, P: Frame> Drop for Bool<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}
