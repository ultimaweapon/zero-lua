use crate::ffi::lua_State;
use crate::state::State;
use std::ops::Deref;

/// Encapsulates [`State`] passed to `lua_CFunction`.
pub struct LocalState(State);

impl LocalState {
    #[inline(always)]
    pub(super) unsafe fn new(state: *mut lua_State) -> Self {
        Self(State::new(state))
    }
}

impl Deref for LocalState {
    type Target = State;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
