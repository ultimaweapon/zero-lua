use super::Frame;
use crate::state::State;
use std::ffi::c_int;
use std::num::NonZero;
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

/// Represents a value in a virtual frame.
///
/// # Safety
/// [`FrameValue::N`] must be correct for this type.
pub unsafe trait FrameValue<'a, P: Frame> {
    const N: NonZero<u8>;
}
