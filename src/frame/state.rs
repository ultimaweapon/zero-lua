use crate::ffi::lua_State;
use std::ffi::c_int;

/// Provides method to get `lua_State` for a frame.
pub trait FrameState: Sized {
    /// Returns a `lua_State` this frame belong to.
    ///
    /// This is a low-level method. Using the returned `lua_State` incorrectly will violate safety
    /// guarantee of this crate. This does not mark as `unsafe` because invoke this method is safe
    /// but using the returned pointer required unsafe code.
    fn state(&self) -> *mut lua_State;

    /// # Safety
    /// `n` must be greater than zero and `n` values on the top of stack must be owned by the
    /// caller.
    unsafe fn release_values(&mut self, n: c_int);
}
