use std::io::{
    Read,
    Write,
};

use bevy::prelude::*;
use erased_serde::{
    deref_erased_serializer,
    Error,
    Map,
    Ok,
    Seq,
    Struct,
    StructVariant,
    Tuple,
    TupleStruct,
    TupleVariant,
};

use crate::erased_serde::{
    self,
    Deserializer,
    Serialize,
    Serializer,
};

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

pub type BoxedSerializer<'w> = Box<dyn Serializer + 'w>;

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

pub fn into_boxed_serializer<'w, S>(ser: S) -> BoxedSerializer<'w>
where
    S: 'w,
    for<'a> &'a mut S: serde::Serializer,
    for<'a> <&'a mut S as serde::Serializer>::Ok: 'static,
{
    Box::new(into_erased_serializer(ser))
}

pub trait Saver: Send + Sync {
    fn serializer<'w>(&self, writer: &'w mut dyn Write) -> BoxedSerializer<'w>;
}

pub trait Loader: Send + Sync {
    fn deserializer<'r>(&self, reader: &'r mut dyn Read) -> Box<dyn Deserializer + 'r>;
}

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
    fn deserializer<'r>(&self, reader: &'r mut dyn Read) -> Box<dyn Deserializer + 'r> {
        todo!()
    }
}

#[derive(Resource)]
pub struct AppSaver(Box<dyn Saver>);

#[derive(Resource)]
pub struct AppLoader(Box<dyn Loader>);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_erase_serializer() {
        let writer = vec![];

        let ser = into_erased_serializer(rmp_serde::Serializer::new(writer));
    }
}
