pub use self::borrowed::*;
pub use self::key::*;
pub use self::setter::*;

use crate::Frame;

mod borrowed;
mod key;
mod setter;

/// Encapsulates an owned table in the stack.
pub struct Table<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> Table<'a, P> {
    /// # Safety
    /// Top of the stack must be a table.
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }

    /// Calling this method without pushing a value to [`TableSetter`] does nothing.
    ///
    /// Note that the returned [`TableSetter`] only keep the last pushed value.
    pub fn set<K: TableKey>(&mut self, key: K) -> TableSetter<'_, 'a, P, K> {
        unsafe { TableSetter::new(self, key) }
    }
}

impl<'a, P: Frame> Drop for Table<'a, P> {
    fn drop(&mut self) {
        unsafe { self.0.release_items(1) };
    }
}
