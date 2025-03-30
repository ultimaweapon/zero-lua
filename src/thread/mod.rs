pub use self::context::*;
pub use self::handle::*;

use crate::Frame;
use crate::ffi::{engine_pop, lua_State, lua54_newstate, zl_close, zl_getextraspace, zl_newthread};
use std::ffi::c_int;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::rc::Rc;

mod context;
mod handle;

/// Encapsulates a `lua_State` created by `lua_newstate`.
pub struct Lua {
    state: *mut lua_State,
    _phantom: PhantomPinned,
}

impl Lua {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            state: lua54_newstate(),
            _phantom: PhantomPinned,
        }
    }

    pub fn spawn(self: &Pin<Rc<Self>>) -> ThreadHandle {
        // Increase main thread references.
        let ptr = unsafe { zl_getextraspace(self.state).cast::<*const Self>() };
        let val = unsafe { ptr.read() };

        if val.is_null() {
            unsafe { ptr.write(Rc::into_raw(Pin::into_inner_unchecked(self.clone()))) };
        } else {
            unsafe { Rc::increment_strong_count(val) };
        }

        // Create thread.
        let td = unsafe { zl_newthread(self.state) };

        unsafe { ThreadHandle::new(td, self.state) }
    }
}

impl Drop for Lua {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { zl_close(self.state) };
    }
}

impl Frame for Lua {
    #[inline(always)]
    fn state(&self) -> *mut lua_State {
        self.state
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { engine_pop(self.state, n) };
    }
}
