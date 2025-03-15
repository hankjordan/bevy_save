use std::io::{
    Read,
    Write,
};

use bevy_save::{
    prelude::*,
    Error,
};
use serde::{
    de::DeserializeSeed,
    Serialize,
};

pub struct RONFormat;

impl Format for RONFormat {
    fn extension() -> &'static str {
        ".ron"
    }

    fn serialize<W: Write, T: Serialize>(writer: W, value: &T) -> Result<(), Error> {
        let mut ser = ron::Serializer::new(writer, Some(ron::ser::PrettyConfig::default()))
            .map_err(Error::saving)?;

        value.serialize(&mut ser).map_err(Error::saving)
    }

    fn deserialize<R: Read, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
        reader: R,
        seed: S,
    ) -> Result<T, Error> {
        ron::options::Options::default()
            .from_reader_seed(reader, seed)
            .map_err(Error::loading)
    }
}

// TODO
fn main() {}
