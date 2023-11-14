use std::io::{
    Read,
    Write,
};

use serde::{
    Deserializer,
    Serializer,
};

// Saver / Loader |----------------------------------------------------------------------------------------------------

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

/// Handles serialization of save data.
pub trait Saver {
    /// The type which can be used as a [`Serializer`].
    type Serializer<W: Write>: AsSerializer;

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
    fn build<W: Write>(writer: W) -> Self::Serializer<W>;
}

/// Handles deserialization of save data.
pub trait Loader {
    /// The type which can be used as a [`Deserializer`].
    type Deserializer<R: Read>: AsDeserializer;

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
    fn build<R: Read>(reader: R) -> Self::Deserializer<R>;
}

// Saver / Loader Implementations |------------------------------------------------------------------------------------

/// An implementation of [`Saver`] that uses [`rmp_serde::Serializer`].
pub struct RMPSaver;

impl Saver for RMPSaver {
    type Serializer<W: Write> = rmp_serde::Serializer<W, rmp_serde::config::DefaultConfig>;

    fn build<W: Write>(writer: W) -> Self::Serializer<W> {
        rmp_serde::Serializer::new(writer)
    }
}

/// An implementation of [`Loader`] that uses [`rmp_serde::Deserializer`].
pub struct RMPLoader;

impl Loader for RMPLoader {
    type Deserializer<R: Read> =
        rmp_serde::Deserializer<rmp_serde::decode::ReadReader<R>, rmp_serde::config::DefaultConfig>;

    fn build<R: Read>(reader: R) -> Self::Deserializer<R> {
        rmp_serde::Deserializer::new(reader)
    }
}

/// An implementation of [`Saver`] that uses [`serde_json::Serializer`].
pub struct JSONSaver;

impl Saver for JSONSaver {
    type Serializer<W: Write> =
        serde_json::Serializer<W, serde_json::ser::PrettyFormatter<'static>>;

    fn build<W: Write>(writer: W) -> Self::Serializer<W> {
        serde_json::Serializer::pretty(writer)
    }
}

/// An implementation of [`Loader`] that uses [`serde_json::Deserializer`].
pub struct JSONLoader;

impl Loader for JSONLoader {
    type Deserializer<R: Read> = serde_json::Deserializer<serde_json::de::IoRead<R>>;

    fn build<R: Read>(reader: R) -> Self::Deserializer<R> {
        serde_json::Deserializer::from_reader(reader)
    }
}

// Defaults |-----------------------------------------------------------------------------------------------------------

/// The [`Saver`] the default [`Pipeline`] will use.
pub type DefaultSaver = RMPSaver;

/// The [`Loader`] the default [`Pipeline`] will use.
pub type DefaultLoader = RMPLoader;

/// The [`Saver`] the default debug [`Pipeline`] will use.
pub type DefaultDebugSaver = JSONSaver;

/// The [`Loader`] the default debug [`Pipeline`] will use.
pub type DefaultDebugLoader = JSONLoader;
