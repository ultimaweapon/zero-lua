pub use self::borrowed::*;
pub use self::frame::*;
pub use self::value::*;

use crate::{Frame, GlobalSetter, Table};
use std::ffi::CStr;
use std::num::NonZero;
use std::panic::RefUnwindSafe;

mod borrowed;
mod frame;
mod value;

/// Strongly typed full userdata.
pub trait UserData: RefUnwindSafe + 'static {
    fn name() -> &'static CStr;

    #[inline(always)]
    fn user_values() -> Option<NonZero<u16>> {
        None
    }

    /// Setup a metatable for the type. This is your only chance to access type's metatable.
    ///
    /// Note that Zero Lua will overwrite the value of `typeid` and `__gc` after this.
    #[inline(always)]
    fn setup_metatable<P: Frame>(t: &mut Table<P>) {
        let _ = t;
    }

    #[inline(always)]
    fn setup_global<P: Frame>(g: GlobalSetter<P, &CStr>) {
        let _ = g;
    }
}

pub(crate) const fn is_boxed<T: UserData>() -> bool {
    align_of::<T>() > align_of::<*mut ()>()
}
