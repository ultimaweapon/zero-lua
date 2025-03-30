pub use self::error::*;
pub use self::frame::*;
pub use self::function::*;
pub use self::global::*;
pub use self::nil::*;
pub use self::option::*;
pub use self::string::*;
pub use self::table::*;
pub use self::thread::*;
pub use self::ty::*;
pub use self::userdata::*;
pub use zl_macros::*;

use self::ffi::{engine_checkstack, engine_pop, zl_getmetafield, zl_tolstring};
use std::borrow::Cow;
use std::ffi::{CStr, c_int};
use std::mem::transmute;
use std::ptr::null_mut;

mod error;
mod ffi;
mod frame;
mod function;
mod global;
mod nil;
mod option;
mod string;
mod table;
mod thread;
mod ty;
mod userdata;

extern crate zl_sys; // Required since no Rust code references this crate.

/// Encapsulates a value in the stack.
#[non_exhaustive]
#[repr(i32)]
pub enum Value<'a, P: Frame> {
    Nil(Nil<'a, P>) = 0,
    String(Str<'a, P>) = 4,
    Table(Table<'a, P>) = 5,
    Function(Function<'a, P>) = 6,
    UserData(UserValue<'a, P>) = 7,
}

impl<'a, P: Frame> Value<'a, P> {
    pub fn ty(&self) -> Type {
        // SAFETY: Value has repr(i32).
        unsafe { transmute((self as *const Self as *const i32).read()) }
    }

    pub fn name(&mut self) -> Cow<'static, CStr> {
        // This is the same algorithm as luaL_typeerror.
        let state = match self {
            Self::Table(v) => v.state(),
            Self::UserData(v) => v.state(),
            _ => return Cow::Borrowed(self.ty().name()),
        };

        // SAFETY: We have an exclusive access to the value, which mean top of the stack always be a
        // value.
        unsafe { engine_checkstack(state, 1) };

        match unsafe { zl_getmetafield(state, -1, c"__name".as_ptr()) } {
            Type::None => unreachable!(),
            Type::Nil => (), // luaL_getmetafield push nothing.
            Type::String => unsafe {
                let v = zl_tolstring(state, -1, null_mut());
                let v = CStr::from_ptr(v).to_owned();

                engine_pop(state, 1);

                return v.into();
            },
            _ => unsafe { engine_pop(state, 1) },
        }

        Cow::Borrowed(self.ty().name())
    }

    pub(crate) unsafe fn from_table<K: TableGetter>(p: &'a mut P, t: c_int, k: K) -> Self {
        unsafe { engine_checkstack(p.state(), 1) };

        match unsafe { k.get_value(p.state(), t) } {
            Type::None => unreachable!(),
            Type::Nil => Self::Nil(unsafe { Nil::new(p) }),
            Type::Boolean => todo!(),
            Type::LightUserData => todo!(),
            Type::Number => todo!(),
            Type::String => Self::String(unsafe { Str::new(p) }),
            Type::Table => Self::Table(unsafe { Table::new(p) }),
            Type::Function => Self::Function(unsafe { Function::new(p) }),
            Type::UserData => Self::UserData(unsafe { UserValue::new(p) }),
            Type::Thread => todo!(),
        }
    }
}
