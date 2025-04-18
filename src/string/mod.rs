use crate::ffi::zl_tolstring;
use crate::{Frame, FromOption, OptionError};
use std::ffi::CStr;
use std::ptr::null_mut;
use std::str::Utf8Error;

/// Represents a string on the top of stack.
pub struct Str<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> Str<'a, P> {
    /// # Safety
    /// Top of the stack must be a string.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }

    /// The returned slice will **not** contain the trailing NUL terminator. However, it is
    /// guarantee there is a NUL past the end.
    ///
    /// Note that the slice may contains NUL.
    #[inline(always)]
    pub fn to_bytes(&mut self) -> &[u8] {
        let mut len = 0;
        let ptr = unsafe { zl_tolstring(self.0.state().get(), -1, &mut len) };

        unsafe { std::slice::from_raw_parts(ptr.cast(), len) }
    }

    pub fn to_option<T: FromOption>(&mut self) -> Result<T, OptionError> {
        let v = self.to_bytes();

        T::from_option(v).ok_or_else(|| OptionError::new(v))
    }

    #[inline(always)]
    pub fn to_c_str(&mut self) -> &CStr {
        unsafe { CStr::from_ptr(zl_tolstring(self.0.state().get(), -1, null_mut())) }
    }

    /// Invoke [`std::str::from_utf8()`] with the result of [`Self::to_bytes()`].
    #[inline(always)]
    pub fn to_str(&mut self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(self.to_bytes())
    }
}

impl<'a, P: Frame> Drop for Str<'a, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}
