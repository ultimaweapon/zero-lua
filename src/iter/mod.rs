use crate::Frame;

/// Represents Lua iterator on the top of stack.
pub struct Iter<'a, P: Frame> {
    parent: &'a mut P,
}

impl<'a, P: Frame> Iter<'a, P> {
    /// # Safety
    /// Top of the stack must be Lua iterator.
    #[inline(always)]
    pub(crate) unsafe fn new(parent: &'a mut P) -> Self {
        Self { parent }
    }
}

impl<P: Frame> Drop for Iter<'_, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.parent.release_values(3) };
    }
}
