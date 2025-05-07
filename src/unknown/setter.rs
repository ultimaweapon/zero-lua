use super::Unknown;
use crate::ffi::{lua_State, zl_replace};
use crate::state::RawState;
use crate::{Frame, PositiveInt};
use std::marker::PhantomData;

/// Provides method to replace the value of [Unknown](super::Unknown).
#[derive(Clone, Copy)]
pub struct UnknownSetter<'a> {
    state: *mut lua_State,
    index: PositiveInt,
    phantom: PhantomData<&'a ()>,
}

impl UnknownSetter<'_> {
    #[inline(always)]
    pub(super) unsafe fn new(state: *mut lua_State, index: PositiveInt) -> Self {
        Self {
            state,
            index,
            phantom: PhantomData,
        }
    }

    /// # Panics
    /// If `v` was constructed from a different `lua_State`.
    #[inline(always)]
    pub fn set<T, P>(self, v: T)
    where
        T: for<'b> Into<Unknown<'b, P>>,
        P: Frame,
    {
        let mut v = v.into();

        assert_eq!(v.state(), self.state);
        unsafe { zl_replace(self.state, self.index.get()) };
        std::mem::forget(v);
    }
}
