pub use self::borrowed::*;
pub use self::frame::*;
pub use self::key::*;

use crate::ffi::zl_pop;
use crate::{Frame, FrameState};
use std::ffi::c_int;

mod borrowed;
mod frame;
mod key;

/// Encapsulates an owned table in the stack.
pub struct Table<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> Table<'a, P> {
    /// # Safety
    /// Top of the stack must be a table.
    #[inline(always)]
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }

    /// Calling this method without pushing a value to [`TableFrame`] does nothing.
    ///
    /// Note that the returned [`TableFrame`] only keep the last pushed value.
    pub fn set<K: TableSetter>(&mut self, key: K) -> TableFrame<Self, K> {
        unsafe { TableFrame::new(self, -2, key) }
    }
}

impl<P: Frame> Drop for Table<'_, P> {
    fn drop(&mut self) {
        unsafe { self.0.release_values(1) };
    }
}

impl<P: Frame> FrameState for Table<'_, P> {
    type State = P::State;

    fn state(&mut self) -> &mut Self::State {
        self.0.state()
    }

    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.state().get(), n) };
    }
}
