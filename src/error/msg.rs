use std::borrow::Cow;
use std::ffi::CStr;

/// Encapsulates an error message.
pub enum ErrorMsg {
    Static(&'static CStr),
    Dynamic(Vec<u8>),
}

impl From<&'static CStr> for ErrorMsg {
    fn from(value: &'static CStr) -> Self {
        Self::Static(value)
    }
}

impl From<String> for ErrorMsg {
    fn from(value: String) -> Self {
        Self::Dynamic(value.into_bytes())
    }
}

impl From<ErrorMsg> for Cow<'static, [u8]> {
    fn from(value: ErrorMsg) -> Self {
        let mut v = match value {
            ErrorMsg::Static(v) => return Cow::Borrowed(v.to_bytes_with_nul()),
            ErrorMsg::Dynamic(v) => v,
        };

        v.push(0);

        Cow::Owned(v)
    }
}
