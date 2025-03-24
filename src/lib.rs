pub use self::error::*;
pub use self::frame::*;
pub use self::function::*;
pub use self::global::*;
pub use self::nil::*;
pub use self::state::*;
pub use self::string::*;
pub use self::table::*;
pub use self::ty::*;

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
pub enum Value<'a, P: Frame> {
    Nil(Nil<'a, P>),
    String(String<'a, P>),
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
}
