pub use self::value::*;

use crate::{Frame, GlobalSetter, Table};
use std::ffi::CStr;
use std::panic::RefUnwindSafe;

mod value;

/// Strongly typed Lua user data.
pub trait UserData: RefUnwindSafe + 'static {
    fn name() -> &'static CStr;

    /// Setup a metatable for the type. This is your only chance to access type's metatable.
    ///
    /// Note that Zero Lua will overwrite the value of `typeid` and `__gc` after this.
    fn setup_metatable<P: Frame>(t: &mut Table<P>) {
        let _ = t;
    }

    fn setup_global<P: Frame>(g: GlobalSetter<P, &CStr>) {
        let _ = g;
    }
}

pub(crate) const fn is_boxed<T: UserData>() -> bool {
    align_of::<T>() > align_of::<*mut ()>()
}
