use crate::state::State;
use std::ffi::c_int;
use std::ops::DerefMut;

/// Provides method to get `lua_State` for a frame.
pub trait FrameState: Sized {
    type State: DerefMut<Target = State>;

    fn state(&mut self) -> &mut Self::State;

    /// # Safety
    /// `n` must be greater than zero and `n` values on the top of stack must be owned by the
    /// caller.
    unsafe fn release_values(&mut self, n: c_int);
}
