use crate::Type;
use crate::ffi::{engine_setfield, lua_State, lua54_getfield, lua54_geti, lua54_seti};
use std::ffi::{CStr, c_int};

/// Key to lookup for a value in the table.
pub trait TableKey {
    unsafe fn get(&self, state: *mut lua_State, table: c_int) -> Type;
    unsafe fn set(&self, state: *mut lua_State, table: c_int);
}

impl TableKey for i64 {
    unsafe fn get(&self, state: *mut lua_State, table: c_int) -> Type {
        unsafe { lua54_geti(state, table, *self) }
    }

    unsafe fn set(&self, state: *mut lua_State, table: c_int) {
        unsafe { lua54_seti(state, table, *self) };
    }
}

impl TableKey for &CStr {
    unsafe fn get(&self, state: *mut lua_State, table: c_int) -> Type {
        unsafe { lua54_getfield(state, table, self.as_ptr()) }
    }

    unsafe fn set(&self, state: *mut lua_State, table: c_int) {
        unsafe { engine_setfield(state, table, self.as_ptr()) };
    }
}
