use crate::ffi::lua_State;
use crate::state::State;
use std::ops::{Deref, DerefMut};

/// Encapsulates a [`State`] that can call into async function.
pub struct AsyncState(State);

impl AsyncState {
    pub(super) unsafe fn new(s: *mut lua_State) -> Self {
        Self(State::new(s))
    }
}

impl Deref for AsyncState {
    type Target = State;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AsyncState {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
