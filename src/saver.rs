use std::io::{
    Read,
    Write,
};

use serde::{
    Deserializer,
    Serializer,
};

// Traits |------------------------------------------------------------------------------------------------------------

/// Implemented for all types whose mutable reference is a [`Serializer`].
pub trait AsSerializer {
    /// The [`Serializer`] type.
    type Output<'a>: Serializer + 'a
    where
        Self: 'a;

    /// Coerce the type into a mutable reference.
    fn as_serializer(&mut self) -> Self::Output<'_>;
}

impl<T> AsSerializer for T
where
    for<'a> &'a mut T: Serializer,
{
    type Output<'a> = &'a mut T where Self: 'a;

    fn as_serializer(&mut self) -> Self::Output<'_> {
        self
    }
}

/// Implemented for all types whose mutable reference is a [`Deserializer`].
pub trait AsDeserializer {
    /// The [`Deserializer`] type.
    type Output<'a>: for<'de> Deserializer<'de> + 'a
    where
        Self: 'a;

    /// Coerce the type into a mutable reference.
    fn as_deserializer(&mut self) -> Self::Output<'_>;
}

impl<T> AsDeserializer for T
where
    for<'a, 'de> &'a mut T: Deserializer<'de>,
{
    type Output<'a> = &'a mut T where Self: 'a;

    fn as_deserializer(&mut self) -> Self::Output<'_> {
        self
    }
}

/// Handles serialization and deserialization of save data.
pub trait Format {
    /// The type that will be used as a [`Serializer`].
    type Serializer<W: Write>: AsSerializer;
    /// The type that will be used as a [`Deserializer`].
    type Deserializer<R: Read>: AsDeserializer;

    /// The file extension used by the format.
    ///
    /// Defaults to `.sav`.
    fn extension() -> &'static str {
        ".sav"
    }

    /// Builds a [`Serializer`] from the given writer.
    ///
    /// # Example
    /// ```
    /// # use bevy_save::prelude::*;
    /// impl Saver for RMPSaver {
    ///     type Serializer<W: Write> = rmp_serde::Serializer<W, rmp_serde::config::DefaultConfig>;
    ///
    ///     fn build<W: Write>(writer: W) -> Self::Serializer<W> {
    ///         rmp_serde::Serializer::new(writer)
    ///     }
    /// }
    /// ```
    fn serializer<W: Write>(writer: W) -> Self::Serializer<W>;

    /// Builds a [`Deserializer`] from the given reader.
    ///
    /// # Example
    /// ```
    /// # use bevy_save::prelude::*;
    /// impl Loader for RMPLoader {
    ///     type Deserializer<R: Read> =
    ///         rmp_serde::Deserializer<rmp_serde::decode::ReadReader<R>, rmp_serde::config::DefaultConfig>;
    ///
    ///     fn build<R: Read>(reader: R) -> Self::Deserializer<R> {
    ///         rmp_serde::Deserializer::new(reader)
    ///     }
    /// }
    /// ```
    fn deserializer<R: Read>(reader: R) -> Self::Deserializer<R>;
}

// Implementations |---------------------------------------------------------------------------------------------------

/// An implementation of [`Format`] that uses [`rmp_serde`].
pub struct RMPFormat;

impl Format for RMPFormat {
    type Serializer<W: Write> = rmp_serde::Serializer<W, rmp_serde::config::DefaultConfig>;
    type Deserializer<R: Read> =
        rmp_serde::Deserializer<rmp_serde::decode::ReadReader<R>, rmp_serde::config::DefaultConfig>;

    fn extension() -> &'static str {
        ".mp"
    }

    fn serializer<W: Write>(writer: W) -> Self::Serializer<W> {
        rmp_serde::Serializer::new(writer)
    }

    fn deserializer<R: Read>(reader: R) -> Self::Deserializer<R> {
        rmp_serde::Deserializer::new(reader)
    }
}

/// An implementation of [`Format`] that uses [`serde_json`].
pub struct JSONFormat;

impl Format for JSONFormat {
    type Serializer<W: Write> =
        serde_json::Serializer<W, serde_json::ser::PrettyFormatter<'static>>;
    type Deserializer<R: Read> = serde_json::Deserializer<serde_json::de::IoRead<R>>;

    fn extension() -> &'static str {
        ".json"
    }

    fn serializer<W: Write>(writer: W) -> Self::Serializer<W> {
        serde_json::Serializer::pretty(writer)
    }

    fn deserializer<R: Read>(reader: R) -> Self::Deserializer<R> {
        serde_json::Deserializer::from_reader(reader)
    }
}

// Defaults |----------------------------------------------------------------------------------------------------------

/// The [`Format`] the default [`Pipeline`](crate::Pipeline) will use.
pub type DefaultFormat = RMPFormat;

/// The [`Format`] the default [`Pipeline`](crate::Pipeline) will use.
pub type DefaultDebugFormat = JSONFormat;
