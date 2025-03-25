use crate::Type;
use crate::ffi::{engine_setfield, lua_State, lua54_getfield, lua54_geti, lua54_seti};
use std::ffi::{CStr, c_int};
use std::io::Write;

/// Key to lookup for a value in the table.
pub trait TableKey {
    unsafe fn get_value(&self, state: *mut lua_State, table: c_int) -> Type;
    unsafe fn set_value(&self, state: *mut lua_State, table: c_int);
    fn display_to(&self, dst: &mut Vec<u8>);
}

impl TableKey for i64 {
    unsafe fn get_value(&self, state: *mut lua_State, table: c_int) -> Type {
        unsafe { lua54_geti(state, table, *self) }
    }

    unsafe fn set_value(&self, state: *mut lua_State, table: c_int) {
        unsafe { lua54_seti(state, table, *self) };
    }

    fn display_to(&self, dst: &mut Vec<u8>) {
        write!(dst, "{}", self).unwrap();
    }
}

impl TableKey for &CStr {
    unsafe fn get_value(&self, state: *mut lua_State, table: c_int) -> Type {
        unsafe { lua54_getfield(state, table, self.as_ptr()) }
    }

    unsafe fn set_value(&self, state: *mut lua_State, table: c_int) {
        unsafe { engine_setfield(state, table, self.as_ptr()) };
    }

    fn display_to(&self, dst: &mut Vec<u8>) {
        dst.push(b'\'');
        dst.extend_from_slice(self.to_bytes());
        dst.push(b'\'');
    }
}
