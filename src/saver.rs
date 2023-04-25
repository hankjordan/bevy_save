use std::io::{
    Read,
    Write,
};

use bevy::prelude::*;

use crate::{
    erased_serde::Error,
    prelude::*,
};

// Writer |------------------------------------------------------------------------------------------------------------

/// A borrowed or owned writer.
pub enum Writer<'w> {
    /// Borrowed variant.
    Borrowed(&'w mut dyn Write),

    /// Owned variant.
    Owned(Box<dyn Write + 'w>),
}

impl<'w, W: Write> From<&'w mut W> for Writer<'w> {
    fn from(value: &'w mut W) -> Self {
        Self::Borrowed(value)
    }
}

impl<'w, W: Write + 'w> From<Box<W>> for Writer<'w> {
    fn from(value: Box<W>) -> Self {
        Self::Owned(value)
    }
}

impl<'w> std::ops::Deref for Writer<'w> {
    type Target = dyn Write + 'w;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(wr) => wr,
            Self::Owned(wr) => wr,
        }
    }
}

impl<'w> std::ops::DerefMut for Writer<'w> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Borrowed(wr) => wr,
            Self::Owned(wr) => wr,
        }
    }
}

impl<'w> std::io::Write for Writer<'w> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        (**self).write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        (**self).flush()
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        (**self).write_vectored(bufs)
    }
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        (**self).write_all(buf)
    }
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        (**self).write_fmt(fmt)
    }
}

// Reader |------------------------------------------------------------------------------------------------------------

/// A borrowed or owned reader.
pub enum Reader<'r> {
    /// Borrowed variant.
    Borrowed(&'r mut dyn Read),

    /// Owned variant.
    Owned(Box<dyn Read + 'r>),
}

impl<'r, R: Read> From<&'r mut R> for Reader<'r> {
    fn from(value: &'r mut R) -> Self {
        Self::Borrowed(value)
    }
}

impl<'r, R: Read + 'r> From<Box<R>> for Reader<'r> {
    fn from(value: Box<R>) -> Self {
        Self::Owned(value)
    }
}

impl<'r> std::ops::Deref for Reader<'r> {
    type Target = dyn Read + 'r;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(re) => re,
            Self::Owned(re) => re,
        }
    }
}

impl<'r> std::ops::DerefMut for Reader<'r> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Borrowed(re) => re,
            Self::Owned(re) => re,
        }
    }
}

impl<'r> std::io::Read for Reader<'r> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        (**self).read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        (**self).read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        (**self).read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        (**self).read_exact(buf)
    }
}

// Saver / Loader |----------------------------------------------------------------------------------------------------

/// Handles serialization of save data.
///
/// Use [`AppSaver`] to override the [`Saver`] that `bevy_save` uses for saving snapshots.
pub trait Saver: Send + Sync + 'static {
    /// Build a serializer trait object from the given writer.
    ///
    /// # Example
    /// ```
    /// # use bevy_save::prelude::*;
    /// pub struct RMPSaver;
    ///
    /// impl Saver for RMPSaver {
    ///     fn serializer<'w>(&self, writer: Writer<'w>) -> IntoSerializer<'w> {
    ///         IntoSerializer::erase(rmp_serde::Serializer::new(writer))
    ///     }
    /// }
    /// ```
    fn serializer<'w>(&self, writer: Writer<'w>) -> IntoSerializer<'w>;
}

/// Handles deserialization of save data.
///
/// Use [`AppLoader`] to override the [`Loader`] that `bevy_save` uses for loading snapshots.
pub trait Loader: Send + Sync + 'static {
    /// Build a deserializer trait object from the given reader.
    /// 
    /// # Example
    /// ```
    /// # use bevy_save::prelude::*;
    /// pub struct RMPLoader;
    /// 
    /// impl Loader for RMPLoader {
    ///     fn deserializer<'r, 'de>(&self, reader: Reader<'r>) -> IntoDeserializer<'r, 'de> {
    ///         IntoDeserializer::erase(rmp_serde::Deserializer::new(reader))
    ///     }
    /// }
    /// ```
    fn deserializer<'r, 'de>(&self, reader: Reader<'r>) -> IntoDeserializer<'r, 'de>;
}

// Saver / Loader Implementations |------------------------------------------------------------------------------------

/// An implementation of [`Saver`] that uses [`rmp_serde::Serializer`].
pub struct RMPSaver;

impl Saver for RMPSaver {
    fn serializer<'w>(&self, writer: Writer<'w>) -> IntoSerializer<'w> {
        IntoSerializer::erase(rmp_serde::Serializer::new(writer))
    }
}

/// An implementation of [`Loader`] that uses [`rmp_serde::Deserializer`].
pub struct RMPLoader;

impl Loader for RMPLoader {
    fn deserializer<'r, 'de>(&self, reader: Reader<'r>) -> IntoDeserializer<'r, 'de> {
        IntoDeserializer::erase(rmp_serde::Deserializer::new(reader))
    }
}

// Resources |---------------------------------------------------------------------------------------------------------

/// The App's [`Saver`].
///
/// `bevy_save` will use this when saving snapshots.
#[derive(Resource)]
pub struct AppSaver(Box<dyn Saver>);

impl AppSaver {
    /// Create a new [`AppSaver`] from the given [`Saver`].
    pub fn new<S: Saver>(saver: S) -> Self {
        Self(Box::new(saver))
    }

    /// Override the current [`Saver`].
    pub fn set<S: Saver>(&mut self, saver: S) {
        self.0 = Box::new(saver);
    }

    /// Returns the current [`Saver`] serializer.
    pub fn serializer<'w, W>(&self, writer: W) -> IntoSerializer<'w>
    where
        W: Into<Writer<'w>>,
    {
        self.0.serializer(writer.into())
    }

    /// Serialize the value to the given writer using the current [`Saver`].
    ///
    /// # Errors
    /// - See [`Error`]
    pub fn serialize<'w, T, W>(&self, value: &T, writer: W) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
        W: Into<Writer<'w>>,
    {
        value.serialize(&mut self.serializer(writer)).map(|_| ())
    }
}

impl Default for AppSaver {
    fn default() -> Self {
        Self::new(RMPSaver)
    }
}

/// The App's [`Loader`].
///
/// `bevy_save` will use this when loading snapshots.
#[derive(Resource)]
pub struct AppLoader(Box<dyn Loader>);

impl AppLoader {
    /// Create a new [`AppLoader`] from the given [`Loader`].
    pub fn new<L: Loader>(loader: L) -> Self {
        Self(Box::new(loader))
    }

    /// Override the current [`Loader`].
    pub fn set<L: Loader>(&mut self, loader: L) {
        self.0 = Box::new(loader);
    }

    /// Returns the current [`Loader`] deserializer.
    pub fn deserializer<'r, 'de: 'r, R>(&self, reader: R) -> IntoDeserializer<'r, 'de>
    where
        R: Into<Reader<'r>>,
    {
        self.0.deserializer(reader.into())
    }

    /// Deserialize the type `T` from the given reader using the current [`Loader`].
    ///
    /// # Errors
    /// - See [`Error`]
    pub fn deserialize<'r, 'de: 'r, T, R>(&self, reader: R) -> Result<T, Error>
    where
        T: serde::Deserialize<'de>,
        R: Into<Reader<'r>>,
    {
        T::deserialize(&mut self.deserializer(reader))
    }
}

impl Default for AppLoader {
    fn default() -> Self {
        Self::new(RMPLoader)
    }
}
