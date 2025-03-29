use crate::Frame;

/// Represents a user data in a frame.
pub struct UserValue<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> UserValue<'a, P> {
    /// # Safety
    /// Top of the stack must be a strongly typed user data.
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }
}

impl<'a, P: Frame> Drop for UserValue<'a, P> {
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}
