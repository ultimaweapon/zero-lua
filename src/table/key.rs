use crate::Type;
use crate::ffi::{engine_setfield, lua_State, lua54_getfield, lua54_geti, lua54_seti};
use std::ffi::{CStr, c_int};
use std::fmt::{Display, Formatter};

/// Key to lookup for a value in the table.
pub trait TableKey {
    unsafe fn get_value(&self, state: *mut lua_State, table: c_int) -> Type;
    unsafe fn set_value(&self, state: *mut lua_State, table: c_int);
    fn display(&self) -> impl Display + '_;
}

impl TableKey for i64 {
    unsafe fn get_value(&self, state: *mut lua_State, table: c_int) -> Type {
        unsafe { lua54_geti(state, table, *self) }
    }

    unsafe fn set_value(&self, state: *mut lua_State, table: c_int) {
        unsafe { lua54_seti(state, table, *self) };
    }

    fn display(&self) -> impl Display + '_ {
        IntDisplay(*self)
    }
}

impl TableKey for &CStr {
    unsafe fn get_value(&self, state: *mut lua_State, table: c_int) -> Type {
        unsafe { lua54_getfield(state, table, self.as_ptr()) }
    }

    unsafe fn set_value(&self, state: *mut lua_State, table: c_int) {
        unsafe { engine_setfield(state, table, self.as_ptr()) };
    }

    fn display(&self) -> impl Display + '_ {
        StrDisplay(*self)
    }
}

/// Implementation of [`Display`] to display integer key.
struct IntDisplay(i64);

impl Display for IntDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Implementation of [`Display`] to display string key.
struct StrDisplay<'a>(&'a CStr);

impl<'a> Display for StrDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.0.to_string_lossy())
    }
}
