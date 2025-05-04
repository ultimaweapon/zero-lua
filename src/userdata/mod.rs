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

pub(crate) const TYPE_ID: &CStr = c"typeid";

pub(crate) const fn is_boxed<T: UserType>() -> bool {
    align_of::<T>() > align_of::<*mut ()>()
}

/// Strongly typed full userdata.
///
/// Note that the type that implement this trait **must** be registered with
/// [`Frame::register_ud()`] before its value can be pushed into Lua.
///
/// This trait has a derive macro to generate a simple implementation for types that need to be
/// passed between Rust and Lua but can't construct or interact with it on Lua side. Zero Lua also
/// provides [class](zl_macros::class()) attribute to generate this implementation that can be
/// constructed or interact with in on Lua side.
pub trait UserType: RefUnwindSafe + 'static {
    fn name() -> &'static CStr;

    /// Returns the number of user values for this type.
    ///
    /// This will be passed as `nuvalue` when Zero Lua call
    /// [lua_newuserdatauv](https://www.lua.org/manual/5.4/manual.html#lua_newuserdatauv).
    #[inline(always)]
    fn user_values() -> Option<NonZero<u16>> {
        None
    }

    /// Setup this type.
    ///
    /// Note that Zero Lua will panic if implementation set `typeid` or `__gc` to `meta`.
    ///
    /// This is your only chance to access type's metatable.
    #[inline(always)]
    fn setup<P: Frame>(meta: &mut Table<P>) {
        let _ = meta;
    }

    #[inline(always)]
    fn register<P: Frame>(g: GlobalSetter<P, &CStr>) {
        let _ = g;
    }
}
