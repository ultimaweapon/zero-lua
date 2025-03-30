pub use self::context::*;

use crate::Frame;
use crate::ffi::{engine_pop, lua_State, lua54_newstate, zl_close};
use std::ffi::c_int;

mod context;

/// Encapsulates a `lua_State` created by `lua_newstate`.
pub struct Lua(*mut lua_State);

impl Lua {
    #[inline(always)]
    pub fn new() -> Self {
        Self(lua54_newstate())
    }
}

impl Drop for Lua {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { zl_close(self.0) };
    }
}

impl Frame for Lua {
    #[inline(always)]
    fn state(&self) -> *mut lua_State {
        self.0
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { engine_pop(self.0, n) };
    }
}
