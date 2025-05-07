use crate::ffi::{zl_pop, zl_tolstring};
use crate::state::RawState;
use crate::{Frame, FromOption, OptionError, Unknown};
use std::ffi::{CStr, c_int};
use std::fmt::{Debug, Formatter};
use std::mem::ManuallyDrop;
use std::ops::DerefMut;
use std::ptr::null_mut;
use std::str::Utf8Error;

/// Represents a string on the top of stack.
pub struct Str<'p, P: Frame>(&'p mut P);

impl<'p, P: Frame> Str<'p, P> {
    /// # Safety
    /// Top of the stack must be a string.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'p mut P) -> Self {
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

    #[inline(always)]
    pub fn into_unknown(self) -> Unknown<'p, P> {
        unsafe { Unknown::new(ManuallyDrop::new(self).deref_mut().0) }
    }
}

impl<P: Frame> Drop for Str<'_, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}

impl<P: Frame> RawState for Str<'_, P> {
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

impl<P: Frame> Debug for Str<'_, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("string")
    }
}

impl<'p, P: Frame> From<Str<'p, P>> for Unknown<'p, P> {
    #[inline(always)]
    fn from(value: Str<'p, P>) -> Self {
        value.into_unknown()
    }
}
