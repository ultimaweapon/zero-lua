use crate::ffi::zl_typename;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::ptr::null_mut;

/// Type of Lua value.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    None = -1,
    Nil,
    Boolean,
    LightUserData,
    Number,
    String,
    Table,
    Function,
    UserData,
    Thread,
}

impl Type {
    #[inline(always)]
    pub fn name(self) -> &'static CStr {
        // SAFETY: Lua does not use L.
        unsafe { CStr::from_ptr(zl_typename(null_mut(), self)) }
    }
}

impl Display for Type {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str((*self).into())
    }
}

impl From<Type> for &'static str {
    #[inline(always)]
    fn from(value: Type) -> Self {
        // SAFETY: All type name returned from zl_typename are UTF-8 and has static storage.
        unsafe { std::str::from_utf8_unchecked(value.name().to_bytes()) }
    }
}
