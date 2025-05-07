pub use self::borrowed::*;
pub use self::frame::*;
pub use self::key::*;

use crate::ffi::zl_pop;
use crate::state::RawState;
use crate::{Frame, Unknown};
use std::ffi::c_int;
use std::mem::ManuallyDrop;
use std::ops::DerefMut;

mod borrowed;
mod frame;
mod key;

/// Encapsulates a table on the top of stack.
pub struct Table<'p, P: Frame>(&'p mut P);

impl<'p, P: Frame> Table<'p, P> {
    /// # Safety
    /// Top of the stack must be a table.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'p mut P) -> Self {
        Self(p)
    }

    /// Calling this method without pushing a value to [`TableFrame`] does nothing.
    ///
    /// Note that the returned [`TableFrame`] only keep the last pushed value.
    #[must_use]
    #[inline(always)]
    pub fn set<K: TableSetter>(&mut self, key: K) -> TableFrame<Self, K> {
        unsafe { TableFrame::new(self, -2, key) }
    }

    #[inline(always)]
    pub fn into_unknown(self) -> Unknown<'p, P> {
        unsafe { Unknown::new(ManuallyDrop::new(self).deref_mut().0) }
    }
}

impl<P: Frame> Drop for Table<'_, P> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}

impl<P: Frame> RawState for Table<'_, P> {
    type State = P::State;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        self.0.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.state().get(), n) };
    }
}

impl<'p, P: Frame> From<Table<'p, P>> for Unknown<'p, P> {
    #[inline(always)]
    fn from(value: Table<'p, P>) -> Self {
        value.into_unknown()
    }
}
