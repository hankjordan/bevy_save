use std::io::{
    Read,
    Write,
};

use bevy::prelude::*;
use bevy_save_erased_serde as erased_serde;
use erased_serde::{
    Deserializer,
    Serialize,
    Serializer,
};

pub trait Saver: Send + Sync {
    fn serialize(
        &self,
        writer: &mut dyn Write,
        value: &dyn Serialize,
    ) -> Result<(), erased_serde::Error>;
}

pub trait Loader: Send + Sync {
    fn deserialize(&self, reader: &mut dyn Read) -> Box<dyn Deserializer>;
}

pub struct RMPSaver;

impl Saver for RMPSaver {
    fn serialize(
        &self,
        writer: &mut dyn Write,
        value: &dyn Serialize,
    ) -> Result<(), erased_serde::Error> {
        value
            .erased_serialize(&mut <dyn Serializer>::erase(
                &mut rmp_serde::Serializer::new(writer),
            ))
            .map(|_| ())
    }
}

pub struct RMPLoader;

impl Loader for RMPLoader {
    fn deserialize(&self, _reader: &mut dyn Read) -> Box<dyn Deserializer> {
        todo!()
    }
}

#[derive(Resource)]
pub struct AppSaver(Box<dyn Saver>);

#[derive(Resource)]
pub struct AppLoader(Box<dyn Loader>);
