use std::io::{
    Read,
    Write,
};

use bevy::prelude::*;
use erased_serde::{
    deref_erased_deserializer,
    deref_erased_serializer,
    impl_serializer_for_trait_object,
    Error,
    Map,
    Ok,
    Out,
    Seq,
    Struct,
    StructVariant,
    Tuple,
    TupleStruct,
    TupleVariant,
    Visitor,
};

use crate::erased_serde::{
    self,
    Deserializer,
    Serialize,
    Serializer,
};

// Erased |------------------------------------------------------------------------------------------------------------

/// Helper trait for an erased concrete type.
/// This is used in form of a trait object for keeping
/// something around to (virtually) call the destructor.
trait Erased {}
impl<T> Erased for T {}

// ErasedSerializer |--------------------------------------------------------------------------------------------------

/// A type-erased serializer.
pub struct ErasedSerializer<'w> {
    _guard: Box<dyn Erased + 'w>,
    serializer: Box<dyn Serializer + 'w>,
}

impl<'w> std::ops::Deref for ErasedSerializer<'w> {
    type Target = dyn Serializer + 'w;

    fn deref(&self) -> &Self::Target {
        &self.serializer
    }
}

impl<'w> std::ops::DerefMut for ErasedSerializer<'w> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.serializer
    }
}

deref_erased_serializer!(<'w> Serializer for ErasedSerializer<'w>);
impl_serializer_for_trait_object!(<'a, 'w> serde::Serializer for &'a mut ErasedSerializer<'w>);

/// Convert a type `S` that implements [`serde::Serializer`] for `&mut S` into a trait object.
pub fn into_erased_serializer<'w, S: 'w>(ser: S) -> ErasedSerializer<'w>
where
    for<'a> &'a mut S: serde::Serializer,
    for<'a> <&'a mut S as serde::Serializer>::Ok: 'static,
{
    let mut ser = Box::new(ser);
    let serializer = Box::new(<dyn Serializer>::erase(unsafe {
        &mut *std::ptr::addr_of_mut!(*ser)
    }));

    ErasedSerializer {
        _guard: ser,
        serializer,
    }
}

// ErasedDeserializer |------------------------------------------------------------------------------------------------

/// A type-erased deserializer.
pub struct ErasedDeserializer<'r, 'de: 'r> {
    _guard: Box<dyn Erased + 'r>,
    deserializer: Box<dyn Deserializer<'de> + 'r>,
}

impl<'r, 'de> std::ops::Deref for ErasedDeserializer<'r, 'de> {
    type Target = dyn Deserializer<'de> + 'r;

    fn deref(&self) -> &Self::Target {
        &self.deserializer
    }
}

impl<'r, 'de> std::ops::DerefMut for ErasedDeserializer<'r, 'de> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.deserializer
    }
}

deref_erased_deserializer!(<'r, 'de> Deserializer<'de> for ErasedDeserializer<'r, 'de>);

/// Convert a type `D` that implements [`serde::Deserializer`] for `&mut D` into a trait object.
pub fn into_erased_deserializer<'r, 'de: 'r, D: 'r>(de: D) -> ErasedDeserializer<'r, 'de>
where
    for<'a> &'a mut D: serde::Deserializer<'de>,
{
    let mut de = Box::new(de);
    let deserializer = Box::new(<dyn Deserializer>::erase(unsafe {
        &mut *std::ptr::addr_of_mut!(*de)
    }));

    ErasedDeserializer {
        _guard: de,
        deserializer,
    }
}

// Saver / Loader |----------------------------------------------------------------------------------------------------

/// A borrowed or owned writer.
pub enum Writer<'w> {
    /// Borrowed variant.
    Borrowed(&'w mut dyn Write),

    /// Owned variant.
    Owned(Box<dyn Write + 'w>),
}

impl<'w> std::ops::Deref for Writer<'w> {
    type Target = dyn Write + 'w;

    fn deref(&self) -> &Self::Target {
        match self {
            Writer::Borrowed(wr) => wr,
            Writer::Owned(wr) => wr,
        }
    }
}

impl<'w> std::ops::DerefMut for Writer<'w> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Writer::Borrowed(wr) => wr,
            Writer::Owned(wr) => wr,
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

/// Handles serialization of save data.
///
/// Use [`AppSaver`] to override the [`Saver`] that `bevy_save` uses for saving snapshots.
pub trait Saver: Send + Sync + 'static {
    /// Build a boxed serializer from the given writer.
    fn serializer<'w>(&self, writer: Writer<'w>) -> ErasedSerializer<'w>;
}

/// Handles deserialization of save data.
///
/// Use [`AppLoader`] to override the [`Loader`] that `bevy_save` uses for loading snapshots.
pub trait Loader: Send + Sync + 'static {
    /// Build a boxed deserializer from the given reader.
    fn deserializer<'r, 'de: 'r>(&self, reader: &'r mut dyn Read) -> ErasedDeserializer<'r, 'de>;
}

// Saver / Loader Implementations |------------------------------------------------------------------------------------

/// An implementation of [`Saver`] that uses [`rmp_serde::Serializer`].
pub struct RMPSaver;

impl Saver for RMPSaver {
    fn serializer<'w>(&self, writer: Writer<'w>) -> ErasedSerializer<'w> {
        into_erased_serializer(rmp_serde::Serializer::new(writer))
    }
}

/// An implementation of [`Loader`] that uses [`rmp_serde::Deserializer`].
pub struct RMPLoader;

impl Loader for RMPLoader {
    fn deserializer<'r, 'de: 'r>(&self, reader: &'r mut dyn Read) -> ErasedDeserializer<'r, 'de> {
        into_erased_deserializer(rmp_serde::Deserializer::new(reader))
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

    pub fn serializer<'w, W>(&self, writer: W) -> ErasedSerializer<'w>
    where
        W: Into<Writer<'w>>,
    {
        self.0.serializer(writer.into())
    }

    /// Serialize the value to the given writer using the current [`Saver`].
    ///
    /// # Errors
    /// - See [`erased_serde::Error`]
    pub fn serialize<'w, T, W>(&self, value: &T, writer: W) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
        W: Into<Writer<'w>>,
    {
        value
            .erased_serialize(&mut self.serializer(writer))
            .map(|_| ())
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

    /// Deserialize the type `T` from the given reader using the current [`Loader`].
    ///
    /// # Errors
    /// - See [`erased_serde::Error`]
    pub fn deserialize<'de, T, R>(&self, mut reader: R) -> Result<T, Error>
    where
        T: serde::Deserialize<'de>,
        R: Read,
    {
        erased_serde::deserialize(&mut self.0.deserializer(&mut reader))
    }
}

// Tests |-------------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_erase_serializer() {
        let writer = vec![];

        let _ser = into_erased_serializer(rmp_serde::Serializer::new(writer));
    }
}
