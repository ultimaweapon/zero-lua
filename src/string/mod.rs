use crate::ffi::zl_tolstring;
use crate::{Frame, FromOption, OptionError};
use std::ffi::CStr;
use std::ptr::null_mut;

/// Represents a string on the top of stack.
pub struct Str<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> Str<'a, P> {
    /// # Safety
    /// Top of the stack must be a string.
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }

    /// The returned slice will **not** contain the trailing NUL terminator.
    pub fn to_bytes(&self) -> &[u8] {
        let mut len = 0;
        let ptr = unsafe { zl_tolstring(self.0.state(), -1, &mut len) };

        unsafe { std::slice::from_raw_parts(ptr.cast(), len) }
    }

    pub fn to_option<T: FromOption>(&self) -> Result<T, OptionError> {
        let v = self.to_bytes();

        T::from_option(v).ok_or_else(|| OptionError::new(v))
    }

    pub fn to_c_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(zl_tolstring(self.0.state(), -1, null_mut())) }
    }
}

impl<'a, P: Frame> Drop for Str<'a, P> {
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}
