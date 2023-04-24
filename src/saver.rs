use std::io::{
    Read,
    Write,
};

use bevy::prelude::*;
use erased_serde::{
    deref_erased_deserializer,
    deref_erased_serializer,
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

// ErasedSerializer |--------------------------------------------------------------------------------------------------

struct ErasedSerializer<'w, S: 'w> {
    _guard: Box<S>,
    serializer: Box<dyn Serializer + 'w>,
}

impl<'w, S> std::ops::Deref for ErasedSerializer<'w, S> {
    type Target = dyn Serializer + 'w;

    fn deref(&self) -> &Self::Target {
        &self.serializer
    }
}

impl<'w, S> std::ops::DerefMut for ErasedSerializer<'w, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.serializer
    }
}

deref_erased_serializer!(
    <'w, S> Serializer for ErasedSerializer<'w, S>
    where
        for<'a> &'a mut S: serde::Serializer,
        for<'a> <&'a mut S as serde::Serializer>::Ok: 'static
);

/// A boxed, type-erased serializer.
pub type BoxedSerializer<'w> = Box<dyn Serializer + 'w>;

/// Convert a type `S` that implements [`serde::Serializer`] for `&mut S` into a trait object.
pub fn into_erased_serializer<'w, S>(ser: S) -> impl Serializer + 'w
where
    S: 'w,
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

/// Convert a type `S` that implements [`serde::Serializer`] for `&mut S` into a boxed trait object.
pub fn into_boxed_serializer<'w, S>(ser: S) -> BoxedSerializer<'w>
where
    S: 'w,
    for<'a> &'a mut S: serde::Serializer,
    for<'a> <&'a mut S as serde::Serializer>::Ok: 'static,
{
    Box::new(into_erased_serializer(ser))
}

// ErasedDeserializer |------------------------------------------------------------------------------------------------

struct ErasedDeserializer<'r, 'de: 'r, D: 'r> {
    _guard: Box<D>,
    deserializer: Box<dyn Deserializer<'de> + 'r>,
}

impl<'r, 'de, D> std::ops::Deref for ErasedDeserializer<'r, 'de, D> {
    type Target = dyn Deserializer<'de> + 'r;

    fn deref(&self) -> &Self::Target {
        &self.deserializer
    }
}

impl<'r, 'de, D> std::ops::DerefMut for ErasedDeserializer<'r, 'de, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.deserializer
    }
}

deref_erased_deserializer!(
    <'r, 'de, D> Deserializer<'de> for ErasedDeserializer<'r, 'de, D>
    where
        for<'a> &'a mut D: serde::Deserializer<'de>
);

/// A boxed, type-erased deserializer.
pub type BoxedDeserializer<'r, 'de> = Box<dyn Deserializer<'de> + 'r>;

/// Convert a type `D` that implements [`serde::Deserializer`] for `&mut D` into a trait object.
pub fn into_erased_deserializer<'r, 'de: 'r, D: 'r>(de: D) -> impl Deserializer<'de> + 'r
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

/// Convert a type `D` that implements [`serde::Deserializer`] for `&mut D` into a boxed trait object.
pub fn into_boxed_deserializer<'r, 'de: 'r, D: 'r>(de: D) -> BoxedDeserializer<'r, 'de>
where
    for<'a> &'a mut D: serde::Deserializer<'de>,
{
    Box::new(into_erased_deserializer(de))
}

// Saver / Loader |----------------------------------------------------------------------------------------------------

/// Handles serialization of save data.
/// 
/// Use [`AppSaver`] to override the [`Saver`] that `bevy_save` uses for saving snapshots.
pub trait Saver: Send + Sync + 'static {
    /// Build a boxed serializer from the given writer.
    fn serializer<'w>(&self, writer: &'w mut dyn Write) -> BoxedSerializer<'w>;
}

/// Handles deserialization of save data.
/// 
/// Use [`AppLoader`] to override the [`Loader`] that `bevy_save` uses for loading snapshots.
pub trait Loader: Send + Sync + 'static {
    /// Build a boxed deserializer from the given reader.
    fn deserializer<'r, 'de: 'r>(&self, reader: &'r mut dyn Read) -> BoxedDeserializer<'r, 'de>;
}

// Saver / Loader Implementations |------------------------------------------------------------------------------------

/// An implementation of [`Saver`] that uses [`rmp_serde::Serializer`].
pub struct RMPSaver;

impl Saver for RMPSaver {
    fn serializer<'w>(&self, writer: &'w mut dyn Write) -> BoxedSerializer<'w> {
        into_boxed_serializer(rmp_serde::Serializer::new(writer))
    }
}

/// An implementation of [`Loader`] that uses [`rmp_serde::Deserializer`].
pub struct RMPLoader;

impl Loader for RMPLoader {
    fn deserializer<'r, 'de: 'r>(&self, reader: &'r mut dyn Read) -> BoxedDeserializer<'r, 'de> {
        into_boxed_deserializer(rmp_serde::Deserializer::new(reader))
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

    /// Serialize the value to the given writer using the current [`Saver`].
    /// 
    /// # Errors
    /// - See [`erased_serde::Error`]
    pub fn serialize<T, W>(&self, value: &T, mut writer: W) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
        W: Write,
    {
        value
            .erased_serialize(&mut self.0.serializer(&mut writer))
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
