use crate::Frame;
use crate::ffi::engine_tostring;
use std::ffi::CStr;

/// Encapsulates an owned string in the stack.
pub struct Str<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> Str<'a, P> {
    /// # Safety
    /// Top of the stack must be a string.
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }

    pub fn get(&self) -> &CStr {
        unsafe { CStr::from_ptr(engine_tostring(self.0.state(), -1)) }
    }
}

impl<'a, P: Frame> Drop for Str<'a, P> {
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}
