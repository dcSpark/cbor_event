//! # CBOR event library
//!
//! [`Deserializer`]: crate::de::Deserializer
//! [`Deserialize`]: crate::de::Deserialize
//! [`Serializer`]: crate::se::Serializer
//! [`Serialize`]: crate::se::Serialize
//! [`Error`]: crate::Error
//! [`Type`]: crate::Type
//! [`Special::Float`]: crate::Special::Float
//! [`Deserializer::float`]: crate::de::Deserializer::float
//! [`Deserializer::float_sz`]: crate::de::Deserializer::float_sz
//! [`Serializer::write_float_sz`]: crate::se::Serializer::write_float_sz
//!
//! `cbor_event` is a minimalist CBOR implementation of the CBOR binary
//! serialisation format. It provides a simple yet efficient way to parse
//! CBOR without the need for an intermediate type representation.
//!
//! Here is the list of supported CBOR primary [`Type`]:
//!
//! - Unsigned and Negative Integers;
//! - Bytes and UTF8 String (of finite and indefinite size);
//! - Array and Map (of finite and indefinite size);
//! - Tag;
//! - Specials (`bool`, `null`, floating points, ...). The raw float readers
//!   accept any width (f16/f32/f64): [`Deserializer::float`] discards the
//!   head width, while the width-preserving pair
//!   [`Deserializer::float_sz`]/[`Serializer::write_float_sz`] round-trips any
//!   encoding byte-exactly (NaN payloads included). [`Special::Float`]
//!   serializes as f64.
//!
//!   The [`Deserialize`]/[`Serialize`] impls for `f32`/`f64` follow the
//!   integer impls: decoding accepts any float head and errors only when
//!   the value does not fit the type ([`Error::ExpectedF32`] instead of
//!   rounding), and encoding writes the smallest width that preserves
//!   the value (RFC 8949 §4.1 preferred serialization). Use the
//!   width-preserving pair above to pin a width.
//!
//! ## Round-trip guarantees
//!
//! Round-trip means two different things here:
//!
//! - value round-trip (value -> bytes -> value): what was serialized
//!   re-decodes bit-exactly, NaN payloads, map entry order and duplicate
//!   keys included. Every layer guarantees this: the typed
//!   [`Serialize`]/[`Deserialize`] impls, the raw
//!   [`Serializer`]/[`Deserializer`] methods, and [`Value`].
//! - byte round-trip (bytes -> decoded form -> bytes): re-encoding
//!   reproduces the identical bytes for any well-formed input, including
//!   non-shortest heads and indefinite-length string chunking. Only the
//!   width-preserving `_sz` pairs guarantee this; every reader has one
//!   ([`Deserializer::unsigned_integer_sz`], `negative_integer_sz`,
//!   [`Deserializer::float_sz`], [`Deserializer::bytes_sz`],
//!   [`Deserializer::text_sz`], `array_sz`, `map_sz`, `tag_sz`, with
//!   their `Serializer::write_*_sz` counterparts), returning the head
//!   width (and for strings the chunk structure) alongside the value.
//!
//! Everything else normalizes the encoding on re-encode: the typed impls
//! write preferred serialization (RFC 8949 §4.1) and [`Value`] normalizes
//! as documented on the type, so non-shortest-form input comes back with
//! different bytes even though the value is identical.
//!
//! [`Deserializer::unsigned_integer_sz`]: crate::de::Deserializer::unsigned_integer_sz
//! [`Deserializer::bytes_sz`]: crate::de::Deserializer::bytes_sz
//! [`Deserializer::text_sz`]: crate::de::Deserializer::text_sz
//! [`Value`]: crate::Value
//!
//! ## Raw deserialisation: [`Deserializer`]
//!
//! Deserialisation works by consuming a `Deserializer` content. To avoid
//! performance issues some objects use a reference to the original
//! source [`Deserializer`] internal buffer.
//!
//! ```
//! use cbor_event::de::*;
//!
//! let vec = vec![0x43, 0x01, 0x02, 0x03];
//! let mut raw = Deserializer::from(vec);
//! let bytes = raw.bytes().unwrap();
//!
//! # assert_eq!(bytes.as_slice(), [1,2,3].as_ref());
//! ```
//!
//! For convenience, we provide the trait [`Deserialize`] to help writing
//! simpler deserializers for your types.
//!
//! ## Serialisation: [`Serializer`]
//!
//! To serialise your objects into CBOR we provide a simple object
//! [`Serializer`]. It is meant to be simple to use and to have limited
//! overhead.
//!
//! ```
//! use cbor_event::se::{Serializer};
//!
//! let mut serializer = Serializer::new_vec();
//! serializer.write_negative_integer(-12)
//!     .expect("write a negative integer");
//!
//! # let bytes = serializer.finalize();
//! # assert_eq!(bytes, [0x2b].as_ref());
//! ```

#![no_std]
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
extern crate alloc;

pub mod de;
mod error;
mod len;
mod macros;
mod result;
pub mod se;
mod types;
mod value;

pub use crate::de::Deserialize;
pub use crate::error::Error;
pub use crate::len::*;
pub use crate::result::Result;
pub use crate::se::Serialize;
pub use crate::types::*;
pub use crate::value::Value;

const MAX_INLINE_ENCODING: u64 = 23;

const CBOR_PAYLOAD_LENGTH_U8: u8 = 24;
const CBOR_PAYLOAD_LENGTH_U16: u8 = 25;
const CBOR_PAYLOAD_LENGTH_U32: u8 = 26;
const CBOR_PAYLOAD_LENGTH_U64: u8 = 27;

/// exported as a convenient function to test the implementation of
/// [`Serialize`] and [`Deserialize`].
///
pub fn test_encode_decode<V: Sized + PartialEq + Serialize + Deserialize>(v: &V) -> Result<bool> {
    let mut se = se::Serializer::new_vec();
    v.serialize(&mut se)?;
    let bytes = se.finalize();

    let mut raw = de::Deserializer::from(bytes);
    let v_ = Deserialize::deserialize(&mut raw)?;

    Ok(v == &v_)
}
