use crate::Type;
use crate::ffi::{engine_setfield, lua_State, lua54_getfield, lua54_geti, lua54_seti, zl_ref};
use std::ffi::{CStr, c_int};
use std::io::Write;

/// Represent a table key.
pub trait TableKey {
    fn display_to(&self, dst: &mut Vec<u8>);
}

impl TableKey for c_int {
    #[inline(always)]
    fn display_to(&self, dst: &mut Vec<u8>) {
        <i64 as TableKey>::display_to(&i64::from(*self), dst);
    }
}

impl TableKey for &mut c_int {
    #[inline(always)]
    fn display_to(&self, dst: &mut Vec<u8>) {
        <c_int as TableKey>::display_to(self, dst);
    }
}

impl TableKey for i64 {
    fn display_to(&self, dst: &mut Vec<u8>) {
        write!(dst, "{}", self).unwrap();
    }
}

impl TableKey for &CStr {
    fn display_to(&self, dst: &mut Vec<u8>) {
        dst.push(b'\'');
        dst.extend_from_slice(self.to_bytes());
        dst.push(b'\'');
    }
}

/// Provides a function to get a value from Lua table.
pub trait TableGetter: TableKey {
    unsafe fn get_value(&self, state: *mut lua_State, table: c_int) -> Type;
}

impl TableGetter for c_int {
    #[inline(always)]
    unsafe fn get_value(&self, state: *mut lua_State, table: c_int) -> Type {
        unsafe { <i64 as TableGetter>::get_value(&i64::from(*self), state, table) }
    }
}

impl TableGetter for i64 {
    #[inline(always)]
    unsafe fn get_value(&self, state: *mut lua_State, table: c_int) -> Type {
        unsafe { lua54_geti(state, table, *self) }
    }
}

impl TableGetter for &CStr {
    #[inline(always)]
    unsafe fn get_value(&self, state: *mut lua_State, table: c_int) -> Type {
        unsafe { lua54_getfield(state, table, self.as_ptr()) }
    }
}

/// Provides a function to set a value to Lua table.
pub trait TableSetter: TableKey {
    unsafe fn set_value(&mut self, state: *mut lua_State, table: c_int);
}

impl TableSetter for &mut c_int {
    #[inline(always)]
    unsafe fn set_value(&mut self, state: *mut lua_State, table: c_int) {
        **self = unsafe { zl_ref(state, table) };
    }
}

impl TableSetter for i64 {
    #[inline(always)]
    unsafe fn set_value(&mut self, state: *mut lua_State, table: c_int) {
        unsafe { lua54_seti(state, table, *self) };
    }
}

impl TableSetter for &CStr {
    #[inline(always)]
    unsafe fn set_value(&mut self, state: *mut lua_State, table: c_int) {
        unsafe { engine_setfield(state, table, self.as_ptr()) };
    }
}
