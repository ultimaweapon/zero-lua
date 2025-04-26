use crate::Frame;
use std::num::NonZero;

/// Type can be converted to Lua value.
///
/// # Safety
/// [`IntoLua::Value`] must be able to handle the values pushed by [`IntoLua::into_lua()`].
pub unsafe trait IntoLua {
    type Value<'a, P: Frame + 'a>: FrameValue<'a, P>;

    /// # Panics
    /// This method may panic if prerequisites for the value is not satisfied.
    fn into_lua<P: Frame>(self, p: &mut P) -> Self::Value<'_, P>;
}

/// Represents a value in a virtual frame.
///
/// # Safety
/// [`FrameValue::N`] must be correct for this type.
pub unsafe trait FrameValue<'a, P: Frame> {
    const N: NonZero<u8>;
}
