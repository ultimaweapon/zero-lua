pub use self::error::*;
pub use self::frame::*;
pub use self::function::*;
pub use self::global::*;
pub use self::nil::*;
pub use self::option::*;
pub use self::state::*;
pub use self::string::*;
pub use self::table::*;
pub use self::ty::*;
pub use zl_macros::*;

use self::ffi::{engine_checkstack, engine_pop, zl_getmetafield};
use std::borrow::Cow;
use std::ffi::CStr;
use std::mem::transmute;

mod error;
mod ffi;
mod frame;
mod function;
mod global;
mod nil;
mod option;
mod state;
mod string;
mod table;
mod ty;

extern crate zl_sys; // Required since no Rust code references this crate.

/// Encapsulates a value in the stack.
#[non_exhaustive]
#[repr(i32)]
pub enum Value<'a, P: Frame> {
    Nil(Nil<'a, P>) = 0,
    String(Str<'a, P>) = 4,
    Table(Table<'a, P>) = 5,
    Function(Function<'a, P>) = 6,
}

impl<'a, P: Frame> Value<'a, P> {
    pub fn ty(&self) -> Type {
        // SAFETY: Value has repr(i32).
        unsafe { transmute((self as *const Self as *const i32).read()) }
    }

    pub fn name(&mut self) -> Cow<'static, CStr> {
        // This is the same algorithm as luaL_typeerror.
        match self {
            Self::Table(v) => {
                unsafe { engine_checkstack(v.state(), 1) };

                match unsafe { zl_getmetafield(v.state(), -1, c"__name".as_ptr()) } {
                    Type::None => unreachable!(),
                    Type::Nil => (), // luaL_getmetafield push nothing.
                    Type::String => return unsafe { Str::new(v).to_c_str().to_owned().into() },
                    _ => unsafe { engine_pop(v.state(), 1) },
                }
            }
            _ => (),
        }

        Cow::Borrowed(self.ty().name())
    }
}
