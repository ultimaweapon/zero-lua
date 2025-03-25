pub use self::error::*;
pub use self::frame::*;
pub use self::function::*;
pub use self::global::*;
pub use self::nil::*;
pub use self::state::*;
pub use self::string::*;
pub use self::table::*;
pub use self::ty::*;

use self::ffi::{engine_checkstack, engine_pop, zl_getmetafield};
use std::borrow::Cow;
use std::ffi::CStr;

mod error;
mod ffi;
mod frame;
mod function;
mod global;
mod nil;
mod state;
mod string;
mod table;
mod ty;

extern crate zl_sys; // Required since no Rust code references this crate.

/// Encapsulates a value in the stack.
#[non_exhaustive]
pub enum Value<'a, P: Frame> {
    Nil(Nil<'a, P>),
    String(Str<'a, P>),
    Table(Table<'a, P>),
    Function(Function<'a, P>),
}

impl<'a, P: Frame> Value<'a, P> {
    pub fn ty(&self) -> Type {
        match self {
            Self::Nil(_) => Type::Nil,
            Self::String(_) => Type::String,
            Self::Table(_) => Type::Table,
            Self::Function(_) => Type::Function,
        }
    }

    pub fn name(&mut self) -> Cow<'static, CStr> {
        // This is the same algorithm as luaL_typeerror.
        match self {
            Self::Table(v) => {
                unsafe { engine_checkstack(v.state(), 1) };

                match unsafe { zl_getmetafield(v.state(), -1, c"__name".as_ptr()) } {
                    Type::None => unreachable!(),
                    Type::Nil => (), // luaL_getmetafield push nothing.
                    Type::String => return unsafe { Str::new(v).get().to_owned().into() },
                    _ => unsafe { engine_pop(v.state(), 1) },
                }
            }
            _ => (),
        }

        Cow::Borrowed(self.ty().name())
    }
}
