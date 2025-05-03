use crate::{Frame, PositiveInt, UserData};

/// Type can be converted to Lua value.
///
/// The purpose of this trait is to provide automatic conversion where manually push is not
/// possible.
///
/// # Safety
/// [`IntoLua::N`] must be correct.
pub unsafe trait IntoLua {
    const N: PositiveInt;

    /// # Panics
    /// This method may panic if prerequisites for the value is not satisfied.
    fn into_lua<P: Frame>(self, p: &mut P);
}

unsafe impl IntoLua for bool {
    const N: PositiveInt = PositiveInt::new(1).unwrap();

    #[inline(always)]
    fn into_lua<P: Frame>(self, p: &mut P) {
        p.push_bool(self);
    }
}

unsafe impl IntoLua for &str {
    const N: PositiveInt = PositiveInt::new(1).unwrap();

    #[inline(always)]
    fn into_lua<P: Frame>(self, p: &mut P) {
        p.push_str(self);
    }
}

unsafe impl IntoLua for &[u8] {
    const N: PositiveInt = PositiveInt::new(1).unwrap();

    #[inline(always)]
    fn into_lua<P: Frame>(self, p: &mut P) {
        p.push_str(self);
    }
}

unsafe impl<T: UserData> IntoLua for T {
    const N: PositiveInt = PositiveInt::new(1).unwrap();

    #[inline(always)]
    fn into_lua<P: Frame>(self, p: &mut P) {
        p.push_ud(self);
    }
}

unsafe impl<T: IntoLua> IntoLua for Option<T> {
    const N: PositiveInt = T::N;

    #[inline(always)]
    fn into_lua<P: Frame>(self, p: &mut P) {
        match self {
            Some(v) => v.into_lua(p),
            None => {
                for _ in 0..Self::N.get() {
                    p.push_nil();
                }
            }
        }
    }
}
