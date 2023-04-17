use std::{
    fs::{
        self,
        File,
    },
    io::{
        BufReader,
        Write,
    },
};

use bevy::{
    ecs::entity::EntityMap,
    prelude::*,
    tasks::IoTaskPool,
};
use rmp_serde::{
    Deserializer,
    Serializer,
};
use serde::{
    de::{
        DeserializeSeed,
        Error,
    },
    Serialize,
};

use crate::{
    get_save_file,
    CloneReflect,
    Rollback,
    Rollbacks,
    SaveableError,
    Snapshot,
    SnapshotDeserializer,
    SnapshotSerializer,
    SAVE_DIR,
};

/// Extension trait that adds save-related methods to Bevy's [`World`].
pub trait WorldSaveableExt: Sized {
    /// Returns a one-to-one [`EntityMap`] for the [`World`].
    fn entity_map(&self) -> EntityMap;

    /// Returns a [`Snapshot`] of the current [`World`] state.
    fn snapshot(&self) -> Snapshot;

    /// Creates a checkpoint for rollback.
    fn checkpoint(&mut self);

    /// Rolls back / forward the [`World`] state.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    fn rollback(&mut self, checkpoints: isize) -> Result<(), SaveableError>;

    /// Rolls back / forward the [`World`] state.
    /// Maps entities to new ids with the [`EntityMap`].
    ///
    /// # Errors
    /// - See [`SaveableError`]
    fn rollback_with_map(
        &mut self,
        checkpoints: isize,
        map: &mut EntityMap,
    ) -> Result<(), SaveableError>;

    /// Analogue of [`serde::Serialize`]
    ///
    /// # Errors
    /// See [`serde::Serialize`]
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;

    /// Analogue of [`serde::Deserialize`], but applies result to current [`World`] instead of creating a new one.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    /// - See [`serde::Deserialize`]
    fn deserialize<'de, D: serde::Deserializer<'de>>(
        &mut self,
        deserializer: D,
    ) -> Result<(), D::Error>;

    /// Analogue of [`serde::Deserialize`], but applies result to current [`World`] instead of creating a new one.
    /// Maps entities to new ids with the [`EntityMap`].
    ///
    /// # Errors
    /// - See [`SaveableError`]
    /// - See [`serde::Deserialize`]
    fn deserialize_with_map<'de, D: serde::Deserializer<'de>>(
        &mut self,
        deserializer: D,
        map: &mut EntityMap,
    ) -> Result<(), D::Error>;

    /// Saves the game state to a named save.
    fn save(&self, name: &str);

    /// Loads the game state from a named save.
    fn load(&mut self, name: &str);

    /// Loads the game state from a named save.
    /// Maps entities to new ids with the [`EntityMap`].
    fn load_with_map(&mut self, name: &str, map: &mut EntityMap);
}

impl WorldSaveableExt for World {
    fn entity_map(&self) -> EntityMap {
        let mut map = EntityMap::default();

        for entity in self.iter_entities() {
            map.insert(entity.id(), entity.id());
        }

        map
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot::from_world(self)
    }

    fn checkpoint(&mut self) {
        let rollback = Rollback::from_world(self);
        let mut state = self.resource_mut::<Rollbacks>();
        state.checkpoint(rollback);
    }

    fn rollback(&mut self, checkpoints: isize) -> Result<(), SaveableError> {
        self.rollback_with_map(checkpoints, &mut self.entity_map())
    }

    fn rollback_with_map(
        &mut self,
        checkpoints: isize,
        map: &mut EntityMap,
    ) -> Result<(), SaveableError> {
        let mut state = self.resource_mut::<Rollbacks>();

        if let Some(snap) = state.rollback(checkpoints).map(|r| r.clone_value()) {
            snap.apply_with_map(self, map)?;
        }

        Ok(())
    }

    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let registry = self.resource::<AppTypeRegistry>();
        let snap = self.snapshot();

        let ser = SnapshotSerializer::new(&snap, registry);

        ser.serialize(serializer)
    }

    fn deserialize<'de, D: serde::Deserializer<'de>>(
        &mut self,
        deserializer: D,
    ) -> Result<(), D::Error> {
        self.deserialize_with_map(deserializer, &mut self.entity_map())
    }

    fn deserialize_with_map<'de, D: serde::Deserializer<'de>>(
        &mut self,
        deserializer: D,
        map: &mut EntityMap,
    ) -> Result<(), D::Error> {
        let registry = self.resource::<AppTypeRegistry>().clone();
        let reg = registry.read();

        let de = SnapshotDeserializer::new(&reg);

        let snap = de.deserialize(deserializer)?;

        snap.apply_with_map(self, map).map_err(D::Error::custom)?;

        Ok(())
    }

    fn save(&self, name: &str) {
        let mut buf = Vec::new();

        self.serialize(&mut Serializer::new(&mut buf))
            .expect("Error serializing save");

        let name = name.to_owned();

        IoTaskPool::get()
            .spawn(async move {
                fs::create_dir_all(&*SAVE_DIR).expect("Could not create save directory");

                File::create(get_save_file(&name))
                    .and_then(|mut file| file.write(buf.as_slice()))
                    .expect("Error writing save to file");
            })
            .detach();
    }

    fn load(&mut self, name: &str) {
        self.load_with_map(name, &mut self.entity_map());
    }

    fn load_with_map(&mut self, name: &str, map: &mut EntityMap) {
        let path = get_save_file(name);
        let file = File::open(path).expect("Could not open save file");

        let mut reader = BufReader::new(file);

        self.deserialize_with_map(&mut Deserializer::new(&mut reader), map)
            .expect("Error deserializing save from file");
    }
}
