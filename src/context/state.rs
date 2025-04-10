use crate::YieldValues;
use crate::ffi::lua_State;
use crate::state::State;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

/// Encapsulates [`State`] passed to `lua_CFunction`.
pub trait LocalState: DerefMut<Target = State> {}

/// Encapsulates [`State`] passed to `lua_CFunction` for non-yieldable function.
pub struct NonYieldable(State);

impl NonYieldable {
    #[inline(always)]
    pub(crate) unsafe fn new(state: *mut lua_State) -> Self {
        Self(State::new(state))
    }
}

impl Deref for NonYieldable {
    type Target = State;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NonYieldable {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl LocalState for NonYieldable {}

/// Encapsulates [`State`] passed to `lua_CFunction` for yieldable function.
pub struct Yieldable {
    state: State,
    values: Rc<Cell<YieldValues>>,
}

impl Yieldable {
    #[inline(always)]
    pub(crate) unsafe fn new(state: *mut lua_State, values: Rc<Cell<YieldValues>>) -> Self {
        Self {
            state: State::new(state),
            values,
        }
    }

    #[inline(always)]
    pub(crate) fn values(this: &Self) -> &Rc<Cell<YieldValues>> {
        &this.values
    }
}

impl Deref for Yieldable {
    type Target = State;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for Yieldable {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl LocalState for Yieldable {}
