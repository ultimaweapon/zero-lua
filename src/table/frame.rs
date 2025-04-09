use super::TableSetter;
use crate::ffi::{engine_pop, lua54_replace};
use crate::{Frame, FrameState};
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

impl<'a, P, K> Drop for TableFrame<'a, P, K>
where
    P: Frame,
    K: TableSetter,
{
    #[inline(always)]
    fn drop(&mut self) {
        if self.has_value {
            unsafe { self.key.set_value(self.parent.state().get(), self.table) };
        }
    }
}

impl<'a, P, K> FrameState for TableFrame<'a, P, K>
where
    P: Frame,
    K: TableSetter,
{
    type State = P::State;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        self.parent.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        // All checks here should be optimized away in most cases since this method and new() forced
        // inline.
        let excess = n - 1;

        if excess > 0 {
            unsafe { engine_pop(self.state().get(), excess) };
        }

        if self.has_value {
            unsafe { lua54_replace(self.state().get(), -2) };
        }

        self.has_value = true;
    }
}
