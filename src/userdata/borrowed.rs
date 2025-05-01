use crate::ffi::{zl_getiuservalue, zl_pop};
use crate::{
    Bool, Frame, FrameState, Function, Nil, PositiveInt, Str, Table, Type, UserValue, Value,
};
use std::ffi::c_int;

/// Encapsulates a full userdata in the stack.
///
/// This kind of userdata either come from function argument or results.
pub struct BorrowedUd<'a, 'b, P: Frame, T> {
    parent: &'a mut P,
    index: PositiveInt,
    ud: &'b T,
}

impl<'a, 'b, P: Frame, T> BorrowedUd<'a, 'b, P, T> {
    #[inline(always)]
    pub(crate) unsafe fn new(parent: &'a mut P, index: PositiveInt, ud: &'b T) -> Self {
        Self { parent, index, ud }
    }

    /// # Panics
    /// If `n` is zero.
    #[inline(always)]
    pub fn get_user_value(&mut self, n: u16) -> Option<Value<Self>> {
        assert!(n > 0);

        let v = match unsafe { zl_getiuservalue(self.state().get(), self.index.get(), n) } {
            Type::None => {
                unsafe { zl_pop(self.state().get(), 1) };
                return None;
            }
            Type::Nil => Value::Nil(unsafe { Nil::new(self) }),
            Type::Boolean => Value::Boolean(unsafe { Bool::new(self) }),
            Type::LightUserData => todo!(),
            Type::Number => todo!(),
            Type::String => Value::String(unsafe { Str::new(self) }),
            Type::Table => Value::Table(unsafe { Table::new(self) }),
            Type::Function => Value::Function(unsafe { Function::new(self) }),
            Type::UserData => Value::UserData(unsafe { UserValue::new(self) }),
            Type::Thread => todo!(),
        };

        Some(v)
    }

    #[inline(always)]
    pub fn into_ud(self) -> &'b T {
        self.ud
    }
}

impl<P: Frame, T> FrameState for BorrowedUd<'_, '_, P, T> {
    type State = P::State;

    #[inline(always)]
    fn state(&mut self) -> &mut Self::State {
        self.parent.state()
    }

    #[inline(always)]
    unsafe fn release_values(&mut self, n: c_int) {
        unsafe { zl_pop(self.state().get(), n) };
    }
}
