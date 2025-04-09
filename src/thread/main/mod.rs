pub(crate) use self::state::*;

use super::AsyncLua;
use crate::FrameState;
use crate::ffi::engine_pop;
use std::ffi::c_int;
use std::pin::Pin;
use std::rc::Rc;

mod state;

/// Encapsulates a `lua_State` created by `lua_newstate`.
pub struct Lua(MainState);

impl Lua {
    /// This function use `luaL_newstate` to create a `lua_State`.
    #[inline(always)]
    pub fn new() -> Self {
        Self(MainState::new())
    }

    pub fn into_async(self) -> Pin<Rc<AsyncLua>> {
        AsyncLua::new(self.0)
    }
}

impl FrameState for Lua {
    type State = MainState;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        &mut self.0
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { engine_pop(self.0.get(), n) };
    }
}
