use crate::ffi::{lua54_newstate, zl_close};
use crate::state::State;
use std::ops::Deref;

/// Encapsulates [`State`] created from `lua_newstate`.
pub struct MainState(State);

impl MainState {
    #[inline(always)]
    pub(super) fn new() -> Self {
        Self(State::new(lua54_newstate()))
    }
}

impl Drop for MainState {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { zl_close(self.0.get()) };
    }
}

impl Deref for MainState {
    type Target = State;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
