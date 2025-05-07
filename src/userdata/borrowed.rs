use super::{TypedUd, UserFrame, UserType};
use crate::ffi::zl_pop;
use crate::state::RawState;
use crate::{Frame, PositiveInt, Value};
use std::ffi::c_int;
use std::num::NonZero;

/// Encapsulates a full userdata somewhere in the stack.
///
/// This kind of userdata either come from function argument or results.
pub struct BorrowedUd<'a, 'b, P: Frame, T> {
    parent: &'a mut P,
    index: PositiveInt,
    ud: &'b T,
}

impl<'a, 'b, P, T> BorrowedUd<'a, 'b, P, T>
where
    P: Frame,
    T: UserType,
{
    #[inline(always)]
    pub(crate) unsafe fn new(parent: &'a mut P, index: PositiveInt, ud: &'b T) -> Self {
        Self { parent, index, ud }
    }

    #[inline(always)]
    pub fn into_ud(self) -> &'b T {
        self.ud
    }
}

impl<P, T> TypedUd for BorrowedUd<'_, '_, P, T>
where
    P: Frame,
    T: UserType,
{
    type Type = T;

    #[inline(always)]
    fn set_uv(&mut self, n: NonZero<u16>) -> Option<UserFrame<Self>> {
        if T::user_values().map(move |v| n <= v).unwrap_or(false) {
            Some(unsafe { UserFrame::new(self, self.index.get(), n) })
        } else {
            None
        }
    }

    #[inline(always)]
    fn get_uv(&mut self, n: NonZero<u16>) -> Option<Value<Self>> {
        unsafe { Value::from_uv(self, self.index.get(), n.get()) }
    }
}

impl<P: Frame, T> RawState for BorrowedUd<'_, '_, P, T> {
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
