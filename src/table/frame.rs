use super::TableSetter;
use crate::Frame;
use crate::ffi::{lua_State, zl_pop, zl_replace};
use crate::state::RawState;
use std::ffi::c_int;

/// Provides [`Frame`] implementation to set a table value.
///
/// Only the first value is used if you push a multiple values (the rest will be discarded) and only
/// the value from the recent push will be used.
pub struct TableFrame<'a, P, K>
where
    P: Frame,
    K: TableSetter,
{
    parent: &'a mut P,
    table: c_int,
    key: K,
    has_value: bool,
}

impl<'a, P, K> TableFrame<'a, P, K>
where
    P: Frame,
    K: TableSetter,
{
    /// # Safety
    /// `table` must be a valid index of the table when this struct is dropped with the value.
    #[inline(always)]
    pub(crate) unsafe fn new(parent: &'a mut P, table: c_int, key: K) -> Self {
        Self {
            parent,
            table,
            key,
            has_value: false,
        }
    }
}

impl<P, K> Drop for TableFrame<'_, P, K>
where
    P: Frame,
    K: TableSetter,
{
    #[inline(always)]
    fn drop(&mut self) {
        if self.has_value {
            unsafe { self.key.set_value(self.parent.state(), self.table) };
        }
    }
}

impl<P, K> RawState for TableFrame<'_, P, K>
where
    P: Frame,
    K: TableSetter,
{
    #[inline(always)]
    fn state(&mut self) -> *mut lua_State {
        self.parent.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        // All checks here should be optimized away in most cases since this method and new() forced
        // inline.
        let excess = n - 1;

        if excess > 0 {
            unsafe { zl_pop(self.state(), excess) };
        }

        if self.has_value {
            unsafe { zl_replace(self.state(), -2) };
        }

        self.has_value = true;
    }
}
