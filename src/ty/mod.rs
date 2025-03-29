use crate::ffi::lua54_typename;
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
    pub fn name(self) -> &'static CStr {
        // SAFETY: Lua does not use L.
        unsafe { CStr::from_ptr(lua54_typename(null_mut(), self)) }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str((*self).into())
    }
}

impl From<Type> for &'static str {
    fn from(value: Type) -> Self {
        // SAFETY: All type name returned from lua54_typename are UTF-8 and has static storage.
        unsafe { std::str::from_utf8_unchecked(value.name().to_bytes()) }
    }
}
