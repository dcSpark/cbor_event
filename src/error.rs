use alloc::string::{FromUtf8Error, String};
use alloc::vec::Vec;
use core::fmt;

use crate::len;
use crate::types::Type;

/// all expected error for cbor parsing and serialising
#[derive(Debug)]
pub enum Error {
    ExpectedU8,
    ExpectedU16,
    ExpectedU32,
    ExpectedU64,
    ExpectedI8,
    ExpectedI16,
    ExpectedI32,
    ExpectedI64,
    /// a float value with no exact f32 representation: sign, precision,
    /// range and NaN payload must all survive the narrowing. Raised by
    /// the [`Deserialize`] impl for `f32`, which accepts any float head
    /// but never rounds; use [`Deserializer::float`] and cast to round
    /// instead.
    ///
    /// The only float variant: decoding into f64 is total (every CBOR
    /// float value is binary64-representable), and rust has no stable
    /// `f16` type to have an impl (rust-lang/rust#116909)
    ///
    /// [`Deserialize`]: crate::de::Deserialize
    /// [`Deserializer::float`]: crate::de::Deserializer::float
    ExpectedF32,
    /// not enough data.
    /// 1st element is the number of bytes available in the current buffer
    /// 2nd element is the total number of bytes needed from the current buffer position.
    /// Exception: `Deserializer::set_position` measures both from the buffer
    /// start instead: `(buffer_len, requested_position)`.
    NotEnough(usize, usize),
    /// Were expecting a different [`Type`]. The first
    /// element is the expected type, the second is the current type.
    Expected(Type, Type),
    ExpectedSetTag,
    /// this may happens when deserialising a [`Deserializer`](crate::de::Deserializer);
    UnknownLenType(u8),
    IndefiniteLenNotSupported(Type),
    WrongLen(u64, len::Len, &'static str),
    InvalidTextError(FromUtf8Error),
    CannotParse(Type, Vec<u8>),
    TrailingData,
    InvalidIndefiniteString,
    InvalidLenPassed(len::Sz),
    InvalidNint(i128),
    /// a Break stop code (`0xff`) where a data item was expected: only
    /// well-formed directly inside an indefinite-length container
    /// (RFC 8949 Appendix C)
    UnexpectedBreak,
    /// a simple value in a non-well-formed form (RFC 8949 §3.3).
    /// On decode: a reserved one-byte codepoint (`0xfc..=0xfe`) or the
    /// two-byte form (`0xf8`) with value < 32. On encode:
    /// `Special::Unassigned(20..=31)` — 20..=23 are assigned (use
    /// `Special::Bool`/`Null`/`Undefined` instead) and 24..=31 have no
    /// well-formed encoding at all
    InvalidSimpleValue(u8),
    CustomError(String),
}
impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Error::InvalidTextError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::Error::*;
        match self {
            ExpectedU8 => write!(f, "Invalid cbor: expected 8bit long unsigned integer"),
            ExpectedU16 => write!(f, "Invalid cbor: expected 16bit long unsigned integer"),
            ExpectedU32 => write!(f, "Invalid cbor: expected 32bit long unsigned integer"),
            ExpectedU64 => write!(f, "Invalid cbor: expected 64bit long unsigned integer"),
            ExpectedI8 => write!(f, "Invalid cbor: expected 8bit long negative integer"),
            ExpectedI16 => write!(f, "Invalid cbor: expected 16bit long negative integer"),
            ExpectedI32 => write!(f, "Invalid cbor: expected 32bit long negative integer"),
            ExpectedI64 => write!(f, "Invalid cbor: expected 64bit long negative integer"),
            ExpectedF32 => write!(
                f,
                "Invalid cbor: expected a float exactly representable in 32 bits"
            ),
            NotEnough(got, exp) => write!(
                f,
                "Invalid cbor: not enough bytes, expect {} bytes but received {} bytes.",
                exp, got
            ),
            Expected(exp, got) => write!(
                f,
                "Invalid cbor: not the right type, expected `{:?}' byte received `{:?}'.",
                exp, got
            ),
            ExpectedSetTag => write!(f, "Invalid cbor: expected set tag"),
            UnknownLenType(byte) => {
                write!(f, "Invalid cbor: not the right sub type: 0b{:05b}", byte)
            }
            IndefiniteLenNotSupported(t) => write!(
                f,
                "Invalid cbor: indefinite length not supported for cbor object of type `{:?}'.",
                t
            ),
            WrongLen(expected_len, actual_len, error_location) => write!(
                f,
                "Invalid cbor: expected tuple '{}' of length {} but got length {:?}.",
                error_location, expected_len, actual_len
            ),
            InvalidTextError(_utf8_error) => {
                write!(f, "Invalid cbor: expected a valid utf8 string text.")
            }
            CannotParse(t, bytes) => write!(
                f,
                "Invalid cbor: cannot parse the cbor object `{:?}' with the following bytes {:?}",
                t, bytes
            ),
            TrailingData => write!(f, "Unexpected trailing data in CBOR"),
            InvalidIndefiniteString => write!(f, "Invalid cbor: Invalid indefinite string format"),
            InvalidLenPassed(sz) => write!(f, "Invalid length for serialization: {:?}", sz),
            UnexpectedBreak => write!(
                f,
                "Invalid cbor: break stop code outside an indefinite-length container"
            ),
            InvalidSimpleValue(v) => write!(
                f,
                "Invalid cbor: non-well-formed encoding of simple value {}",
                v
            ),
            CustomError(err) => write!(f, "Invalid cbor: {}", err),
            InvalidNint(x) => write!(f, "Passed nint {} out of range", x),
        }
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Error::InvalidTextError(error) => Some(error),
            _ => None,
        }
    }
}
