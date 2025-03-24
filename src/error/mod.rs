pub use self::msg::*;

use crate::{Frame, TableKey, Value};
use std::borrow::Cow;
use std::ffi::c_int;

mod msg;

/// Represents an error when Lua function that defined on Rust side fails.
///
/// The message will be silently truncated if its contain a NUL byte.
pub struct Error(ErrorKind);

impl Error {
    /// # Panics
    /// If `arg` is zero or negative.
    pub fn arg_type(arg: c_int, expect: impl Into<ErrorMsg>) -> Self {
        assert!(arg > 0);

        Self(ErrorKind::ArgType(arg, expect.into().into()))
    }

    /// # Panics
    /// If `arg` is zero or negative.
    pub fn arg_from_std(arg: c_int, e: impl std::error::Error) -> Self {
        let mut msg = e.to_string();
        let mut src = e.source();

        while let Some(e) = src {
            use std::fmt::Write;

            write!(msg, " -> {e}").unwrap();
            src = e.source();
        }

        Self::arg(arg, msg)
    }

    /// # Panics
    /// If `arg` is zero or negative.
    pub fn arg(arg: c_int, msg: impl Into<ErrorMsg>) -> Self {
        assert!(arg > 0);

        Self(ErrorKind::Arg(arg, msg.into().into()))
    }

    /// # Panics
    /// If `arg` is zero or negative.
    pub fn arg_table_type<P>(
        arg: c_int,
        expect: impl AsRef<[u8]>,
        key: impl TableKey,
        mut val: Value<P>,
    ) -> Self
    where
        P: Frame,
    {
        use std::io::Write;

        let mut m = Vec::new();

        m.extend_from_slice(expect.as_ref());
        m.extend_from_slice(b" expected on key ");

        write!(m, "{}", key.display()).unwrap();

        m.extend_from_slice(b", got ");
        m.extend_from_slice(val.name().to_bytes());

        Self::arg(arg, ErrorMsg::Dynamic(m))
    }

    /// `msg` are typically concise lowercase sentences without trailing punctuation (e.g. `failed
    /// to open 'foo'`).
    pub fn other(msg: impl Into<ErrorMsg>) -> Self {
        Self(ErrorKind::Other(msg.into().into()))
    }

    /// `msg` are typically concise lowercase sentences without trailing punctuation (e.g. `failed
    /// to open 'foo'`).
    pub fn with_source(msg: impl Into<String>, src: impl std::error::Error) -> Self {
        let mut msg = msg.into();
        let mut src: Option<&dyn std::error::Error> = Some(&src);

        while let Some(e) = src {
            use std::fmt::Write;

            write!(msg, " -> {e}").unwrap();
            src = e.source();
        }

        Self::other(msg)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::other(value)
    }
}

impl<E: std::error::Error> From<(String, E)> for Error {
    fn from(value: (String, E)) -> Self {
        Self::with_source(value.0, value.1)
    }
}

/// Kind of [`Error`].
pub(crate) enum ErrorKind {
    /// # Safety
    /// Second value must null-terminated.
    Arg(c_int, Cow<'static, [u8]>),
    /// # Safety
    /// - First value must be positive.
    /// - Second value must null-terminated.
    ArgType(c_int, Cow<'static, [u8]>),
    /// # Safety
    /// The value must null-terminated.
    Other(Cow<'static, [u8]>),
}

impl From<Error> for ErrorKind {
    fn from(value: Error) -> Self {
        value.0
    }
}
