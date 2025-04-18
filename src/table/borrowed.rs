use super::TableGetter;
use crate::ffi::engine_pop;
use crate::{Frame, FrameState, Value};
use std::ffi::c_int;

/// Encapsulates a borrowed table in the stack.
///
/// This kind of table either come from function argument or results.
pub struct BorrowedTable<'a, P: Frame> {
    parent: &'a mut P,
    index: c_int,
}

impl<'a, P: Frame> BorrowedTable<'a, P> {
    /// # Safety
    /// `index` must be a table.
    #[inline(always)]
    pub(crate) unsafe fn new(parent: &'a mut P, index: c_int) -> Self {
        Self { parent, index }
    }

    #[inline(always)]
    pub fn get<K: TableGetter>(&mut self, key: K) -> Value<Self> {
        unsafe { Value::from_table(self, self.index, key) }
    }
}

impl<'a, P: Frame> FrameState for BorrowedTable<'a, P> {
    type State = P::State;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        self.parent.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { engine_pop(self.state().get(), n) };
    }
}
