use crate::YieldValues;
use crate::ffi::lua_State;
use std::cell::Cell;
use std::rc::Rc;

/// Encapsulates `lua_State` passed to `lua_CFunction`.
pub trait LocalState {
    fn get(&mut self) -> *mut lua_State;
}

/// Encapsulates a `lua_State` passed to `lua_CFunction` for non-yieldable function.
pub struct NonYieldable(*mut lua_State);

impl NonYieldable {
    #[inline(always)]
    pub(crate) unsafe fn new(state: *mut lua_State) -> Self {
        Self(state)
    }
}

impl LocalState for NonYieldable {
    #[inline(always)]
    fn get(&mut self) -> *mut lua_State {
        self.0
    }
}

/// Encapsulates `lua_State` passed to `lua_CFunction` for yieldable function.
pub struct Yieldable {
    state: *mut lua_State,
    values: Rc<Cell<YieldValues>>,
}

impl Yieldable {
    #[inline(always)]
    pub(crate) unsafe fn new(state: *mut lua_State, values: Rc<Cell<YieldValues>>) -> Self {
        Self { state, values }
    }

    #[inline(always)]
    pub(crate) fn values(this: &Self) -> &Rc<Cell<YieldValues>> {
        &this.values
    }
}

impl LocalState for Yieldable {
    #[inline(always)]
    fn get(&mut self) -> *mut lua_State {
        self.state
    }
}
