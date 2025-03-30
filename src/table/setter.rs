use super::TableKey;
use crate::Frame;
use crate::ffi::{engine_pop, lua_State, lua54_replace};
use std::ffi::c_int;

/// Provides [`Frame`] implementation to set a table value.
///
/// Only the first value is used if you push a multiple values (the rest will be discarded) and only
/// the value from the recent push will be used.
pub struct TableSetter<'a, P, K>
where
    P: Frame,
    K: TableKey,
{
    parent: &'a mut P,
    key: K,
    has_value: bool,
}

impl<'a, P, K> TableSetter<'a, P, K>
where
    P: Frame,
    K: TableKey,
{
    /// # Safety
    /// Top of the stack must be a table.
    #[inline(always)]
    pub(crate) unsafe fn new(parent: &'a mut P, key: K) -> Self {
        Self {
            parent,
            key,
            has_value: false,
        }
    }
}

impl<'a, P, K> Drop for TableSetter<'a, P, K>
where
    P: Frame,
    K: TableKey,
{
    #[inline(always)]
    fn drop(&mut self) {
        if self.has_value {
            unsafe { self.key.set_value(self.parent.state(), -2) };
        }
    }
}

impl<'a, P, K> Frame for TableSetter<'a, P, K>
where
    P: Frame,
    K: TableKey,
{
    #[inline(always)]
    fn state(&self) -> *mut lua_State {
        self.parent.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        // All checks here should be optimized away in most cases since this method and new() should
        // be automatically inlined.
        let excess = n - 1;

        if excess > 0 {
            unsafe { engine_pop(self.state(), excess) };
        }

        if self.has_value {
            unsafe { lua54_replace(self.state(), -2) };
        }

        self.has_value = true;
    }
}
