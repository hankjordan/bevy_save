use bevy::prelude::*;
use serde::{
    de::{
        DeserializeSeed,
        Error,
    },
    Serialize,
};

use crate::{
    AppBackend,
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
    /// The applier allows you to customize how the [`Rollback`] will be applied to the [`World`].
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
    /// The applier allows you to customize how the [`Snapshot`] will be applied to the [`World`].
    ///
    /// # Errors
    /// - See [`SaveableError`]
    /// - See [`serde::Deserialize`]
    fn deserialize_applier<'de, D: serde::Deserializer<'de>>(
        &mut self,
        deserializer: D,
    ) -> Result<Applier<Snapshot>, D::Error>;

    /// Saves the game state to a named save.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    /// - See [`serde::Serialize`]
    fn save(&self, name: &str) -> Result<(), SaveableError>;

    /// Loads the game state from a named save.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    /// - See [`serde::Deserialize`]
    fn load(&mut self, name: &str) -> Result<(), SaveableError>;

    /// Loads the game state from a named save.
    /// 
    /// The applier allows you to customize how the [`Snapshot`] will be applied to the [`World`].
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

    fn save(&self, name: &str) -> Result<(), SaveableError> {
        let mut writer = self
            .resource::<AppBackend>()
            .writer(name)
            .map_err(SaveableError::other)?;

        let saver = self.resource::<AppSaver>();

        self.serialize(&mut saver.serializer(&mut writer))
            .map_err(SaveableError::other)?;

        Ok(())
    }

    fn load(&mut self, name: &str) -> Result<(), SaveableError> {
        self.load_applier(name)?.apply()
    }

    fn load_applier(&mut self, name: &str) -> Result<Applier<Snapshot>, SaveableError> {
        let mut reader = self.resource::<AppBackend>().reader(name)?;

        let loader = self.resource::<AppLoader>();

        let applier = self
            .deserialize_applier(&mut loader.deserializer(&mut reader))
            .map_err(SaveableError::other)?;

        Ok(applier)
    }
}
