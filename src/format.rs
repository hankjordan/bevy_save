//! [`Format`] handles serialization and deserialization of application types.

use std::io::{
    Read,
    Write,
};

use serde::{
    Serialize,
    de::DeserializeSeed,
};

use crate::error::Error;

/// Handles serialization and deserialization of save data.
pub trait Format {
    /// The file extension used by the format.
    ///
    /// Defaults to `.sav`.
    fn extension() -> &'static str {
        ".sav"
    }

    /// Serializes a value with the format.
    ///
    /// # Errors
    /// If serialization fails.
    fn serialize<W: Write, T: Serialize>(writer: W, value: &T) -> Result<(), Error>;

    /// Deserializes a value with the format.
    ///
    /// # Errors
    /// If deserialization fails.
    fn deserialize<R: Read, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
        reader: R,
        seed: S,
    ) -> Result<T, Error>;
}

/// An implementation of [`Format`] that uses [`rmp_serde`].
pub struct RMPFormat;

impl Format for RMPFormat {
    fn extension() -> &'static str {
        ".mp"
    }

    fn serialize<W: Write, T: Serialize>(writer: W, value: &T) -> Result<(), Error> {
        let mut ser = rmp_serde::Serializer::new(writer);
        value.serialize(&mut ser).map_err(Error::saving)
    }

    fn deserialize<R: Read, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
        reader: R,
        seed: S,
    ) -> Result<T, Error> {
        let mut de = rmp_serde::Deserializer::new(reader);
        seed.deserialize(&mut de).map_err(Error::loading)
    }
}

/// An implementation of [`Format`] that uses [`serde_json`].
pub struct JSONFormat;

impl Format for JSONFormat {
    fn extension() -> &'static str {
        ".json"
    }

    fn serialize<W: Write, T: Serialize>(writer: W, value: &T) -> Result<(), Error> {
        let mut ser = serde_json::Serializer::pretty(writer);
        value.serialize(&mut ser).map_err(Error::saving)
    }

    fn deserialize<R: Read, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
        reader: R,
        seed: S,
    ) -> Result<T, Error> {
        let mut de = serde_json::Deserializer::from_reader(reader);
        seed.deserialize(&mut de).map_err(Error::loading)
    }
}

/// A reasonable default [`Format`].
pub type DefaultFormat = RMPFormat;

/// A reasonable default debug [`Format`], human-readable.
pub type DefaultDebugFormat = JSONFormat;
