#![doc = include_str!("../README.md")]

pub use self::boolean::*;
pub use self::context::*;
pub use self::convert::*;
pub use self::error::*;
pub use self::frame::*;
pub use self::function::*;
pub use self::global::*;
pub use self::iter::*;
pub use self::nil::*;
pub use self::option::*;
pub use self::string::*;
pub use self::table::*;
pub use self::thread::*;
pub use self::ty::*;
pub use self::unknown::*;
pub use self::userdata::*;
pub use self::util::*;
pub use zl_macros::*;

use self::ffi::{zl_getiuservalue, zl_getmetafield, zl_pop, zl_tolstring};
use self::state::FrameState;
use std::borrow::Cow;
use std::ffi::{CStr, c_int};
use std::mem::transmute;
use std::ptr::null_mut;

mod boolean;
mod context;
mod convert;
mod error;
mod ffi;
mod frame;
mod function;
mod global;
mod iter;
mod nil;
mod option;
mod state;
mod string;
mod table;
mod thread;
mod ty;
mod unknown;
mod userdata;
mod util;

extern crate zl_sys; // Required since no Rust code references this crate.

pub type PanicHandler = dyn Fn(Option<&str>);

/// Allowed chunk type to load.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChunkType {
    Text,
    Binary,
    Both,
}

impl ChunkType {
    const fn to_c_str(self) -> &'static CStr {
        match self {
            Self::Text => c"t",
            Self::Binary => c"b",
            Self::Both => c"bt",
        }
    }
}

/// Encapsulates a value in the stack.
#[non_exhaustive]
#[repr(i32)]
pub enum Value<'a, P: Frame> {
    Nil(Nil<'a, P>) = 0,
    Boolean(Bool<'a, P>) = 1,
    String(Str<'a, P>) = 4,
    Table(Table<'a, P>) = 5,
    Function(Function<'a, P>) = 6,
    UserData(UserData<'a, P>) = 7,
}

impl<'a, P: Frame> Value<'a, P> {
    #[inline(always)]
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
        match unsafe { zl_getmetafield(state.get(), -1, c"__name".as_ptr()) } {
            Type::None => unreachable!(),
            Type::Nil => (), // luaL_getmetafield push nothing.
            Type::String => unsafe {
                let v = zl_tolstring(state.get(), -1, null_mut());
                let v = CStr::from_ptr(v).to_owned();

                zl_pop(state.get(), 1);

                return v.into();
            },
            _ => unsafe { zl_pop(state.get(), 1) },
        }

        Cow::Borrowed(self.ty().name())
    }

    #[inline(always)]
    pub(crate) unsafe fn from_table<K: TableGetter>(p: &'a mut P, t: c_int, k: K) -> Self {
        match unsafe { k.get_value(p.state().get(), t) } {
            Type::None => unreachable!(),
            Type::Nil => Self::Nil(unsafe { Nil::new(p) }),
            Type::Boolean => Self::Boolean(unsafe { Bool::new(p) }),
            Type::LightUserData => todo!(),
            Type::Number => todo!(),
            Type::String => Self::String(unsafe { Str::new(p) }),
            Type::Table => Self::Table(unsafe { Table::new(p) }),
            Type::Function => Self::Function(unsafe { Function::new(p) }),
            Type::UserData => Self::UserData(unsafe { UserData::new(p) }),
            Type::Thread => todo!(),
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn from_uv(p: &'a mut P, d: c_int, v: u16) -> Option<Self> {
        let v = match unsafe { zl_getiuservalue(p.state().get(), d, v) } {
            Type::None => {
                unsafe { zl_pop(p.state().get(), 1) };
                return None;
            }
            Type::Nil => Value::Nil(unsafe { Nil::new(p) }),
            Type::Boolean => Value::Boolean(unsafe { Bool::new(p) }),
            Type::LightUserData => todo!(),
            Type::Number => todo!(),
            Type::String => Value::String(unsafe { Str::new(p) }),
            Type::Table => Value::Table(unsafe { Table::new(p) }),
            Type::Function => Value::Function(unsafe { Function::new(p) }),
            Type::UserData => Value::UserData(unsafe { UserData::new(p) }),
            Type::Thread => todo!(),
        };

        Some(v)
    }
}

#[cfg(not(panic = "unwind"))]
compile_error!("Zero Lua can only be used with unwinding panic.");
