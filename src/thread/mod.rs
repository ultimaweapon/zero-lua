use crate::Frame;
use crate::ffi::{engine_pop, lua_State, lua54_close, lua54_newstate};
use std::ffi::c_int;

/// Encapsulates a `lua_State` created by `lua_newstate`.
pub struct Lua(*mut lua_State);

impl Lua {
    pub fn new() -> Self {
        Self(lua54_newstate())
    }
}

impl Drop for Lua {
    fn drop(&mut self) {
        unsafe { lua54_close(self.0) };
    }
}

impl Frame for Lua {
    fn state(&self) -> *mut lua_State {
        self.0
    }

    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { engine_pop(self.0, n) };
    }
}
