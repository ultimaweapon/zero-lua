pub use self::value::*;

use crate::{Frame, Table};
use std::ffi::CStr;
use std::panic::RefUnwindSafe;

mod value;

/// Strongly typed Lua user data.
pub trait UserData: RefUnwindSafe + 'static {
    fn name() -> &'static CStr;

    /// Note that Zero Lua will overwrite the value of `typeid` and `__gc` after this.
    fn setup_metatable<P: Frame>(t: &mut Table<P>) {
        let _ = t;
    }
}

pub(crate) const fn is_boxed<T: UserData>() -> bool {
    align_of::<T>() > align_of::<*mut ()>()
}
