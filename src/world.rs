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
    prelude::*,
    tasks::IoTaskPool,
};
use rmp_serde::Deserializer;
use serde::{
    de::{
        DeserializeSeed,
        Error,
    },
    Serialize,
};

use crate::{
    get_save_file,
    AppLoader,
    AppSaver,
    Applier,
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
    ///
    /// # Errors
    /// - See [`SaveableError`]
    fn rollback_applier(&mut self, checkpoints: isize) -> Option<Applier<Rollback>>;

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
    ///
    /// # Errors
    /// - See [`SaveableError`]
    /// - See [`serde::Deserialize`]
    fn deserialize_applier<'de, D: serde::Deserializer<'de>>(
        &mut self,
        deserializer: D,
    ) -> Result<Applier<Snapshot>, D::Error>;

    /// Saves the game state to a named save.
    fn save(&self, name: &str);

    /// Loads the game state from a named save.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    /// - See [`serde::Deserialize`]
    fn load(&mut self, name: &str) -> Result<(), SaveableError>;

    /// Loads the game state from a named save.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    /// - See [`serde::Deserialize`]
    fn load_applier(&mut self, name: &str) -> Result<Applier<Snapshot>, SaveableError>;
}

impl WorldSaveableExt for World {
    fn snapshot(&self) -> Snapshot {
        Snapshot::from_world(self)
    }

    fn checkpoint(&mut self) {
        let rollback = Rollback::from_world(self);
        let mut state = self.resource_mut::<Rollbacks>();
        state.checkpoint(rollback);
    }

    fn rollback(&mut self, checkpoints: isize) -> Result<(), SaveableError> {
        self.rollback_applier(checkpoints)
            .map_or(Ok(()), |a| a.apply())
    }

    fn rollback_applier(&mut self, checkpoints: isize) -> Option<Applier<Rollback>> {
        let mut state = self.resource_mut::<Rollbacks>();

        state
            .rollback(checkpoints)
            .map(|r| r.clone_value())
            .map(|snap| snap.into_applier(self))
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
        self.deserialize_applier(deserializer)?
            .apply()
            .map_err(Error::custom)
    }

    fn deserialize_applier<'de, D: serde::Deserializer<'de>>(
        &mut self,
        deserializer: D,
    ) -> Result<Applier<Snapshot>, D::Error> {
        let registry = self.resource::<AppTypeRegistry>().clone();
        let reg = registry.read();

        let de = SnapshotDeserializer::new(&reg);

        let snap = de.deserialize(deserializer)?;

        Ok(snap.into_applier(self))
    }

    fn save(&self, name: &str) {
        let mut buf = Vec::new();

        let saver = self.resource::<AppSaver>();
        self.serialize(&mut saver.serializer(&mut buf))
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

    fn load(&mut self, name: &str) -> Result<(), SaveableError> {
        self.load_applier(name)?.apply()
    }

    fn load_applier(&mut self, name: &str) -> Result<Applier<Snapshot>, SaveableError> {
        let path = get_save_file(name);
        let file = File::open(path).map_err(SaveableError::other)?;

        let mut reader = BufReader::new(file);

        self.deserialize_applier(&mut Deserializer::new(&mut reader))
            .map_err(SaveableError::other)
    }
}
