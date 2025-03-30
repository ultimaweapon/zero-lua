use super::Lua;
use crate::ffi::{ZL_REGISTRYINDEX, lua_State, zl_getextraspace, zl_ref, zl_unref};
use std::ffi::c_int;
use std::rc::Rc;

/// Encapsulates a Lua thread on the registry.
pub struct ThreadHandle {
    state: *mut lua_State,
    index: c_int,
}

impl ThreadHandle {
    /// # Safety
    /// - Top of `parent` must be a Lua thread.
    /// - Main thread references must be increased for this thread.
    pub(crate) unsafe fn new(state: *mut lua_State, parent: *mut lua_State) -> Self {
        let index = unsafe { zl_ref(parent, ZL_REGISTRYINDEX) };

        Self { state, index }
    }
}

impl Drop for ThreadHandle {
    fn drop(&mut self) {
        // Load main thread before remove from registry otherwise Lua GC might free the value.
        let ptr = unsafe { zl_getextraspace(self.state).cast::<*const Lua>() };
        let val = unsafe { ptr.read() };

        unsafe { zl_unref(self.state, ZL_REGISTRYINDEX, self.index) };

        // Decrease main thread references. This must be done as the last thing here since it can
        // free the main thread.
        unsafe { Rc::decrement_strong_count(val) };
    }
}
