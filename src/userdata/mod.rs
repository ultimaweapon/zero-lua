pub use self::value::*;

use crate::ffi::{lua_State, zl_getfield, zl_globalmetatable, zl_pop, zl_touserdata};
use crate::{Frame, GlobalSetter, Table, Type};
use std::any::{TypeId, type_name};
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

/// # Panics
/// If `T` is not registered.
#[inline(never)]
unsafe fn push_metatable<T: UserData>(#[allow(non_snake_case)] L: *mut lua_State) {
    // Get metatable.
    match unsafe { zl_globalmetatable(L, T::name().as_ptr()) } {
        Type::Nil => panic!("{} is not registered", type_name::<T>()),
        Type::Table => (),
        _ => unreachable!(),
    }

    // SAFETY: Checking field type does not really give us 100% safe. The only cases
    // "typeid" is not our value are either:
    //
    // 1. Our user use lua_State wrong.
    // 2. We screw ourself.
    //
    // The first case required unsafe code and the second case is our own bug.
    unsafe { zl_getfield(L, -1, c"typeid".as_ptr()) };

    // SAFETY: TypeId is Copy.
    let ud = unsafe { zl_touserdata(L, -1) };
    let id = unsafe { ud.cast::<TypeId>().read_unaligned() };

    unsafe { zl_pop(L, 1) };

    if id != TypeId::of::<T>() {
        panic!("{} is not registered", type_name::<T>())
    }
}
