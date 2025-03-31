use super::{Lua, ThreadHandle};
use crate::ffi::{lua_State, zl_close, zl_getextraspace, zl_newthread};
use std::marker::PhantomPinned;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::rc::Rc;

/// Provides method to create a Lua thread with ability to call into async function.
pub struct AsyncLua {
    state: *mut lua_State,
    _phantom: PhantomPinned,
}

impl AsyncLua {
    pub(super) fn new(main: Lua) -> Pin<Rc<Self>> {
        let state = ManuallyDrop::new(main).0;

        Rc::pin(Self {
            state,
            _phantom: PhantomPinned,
        })
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

impl Drop for AsyncLua {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { zl_close(self.state) };
    }
}
