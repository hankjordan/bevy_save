use std::{
    fs::{
        self,
        File,
    },
    io::{
        BufReader,
        Write,
    },
    path::PathBuf,
};

use bevy::{
    prelude::*,
    tasks::IoTaskPool,
};
use lazy_static::lazy_static;
use platform_dirs::AppDirs;
use rmp_serde::{
    Deserializer,
    Serializer,
};
use serde::{
    de::DeserializeSeed,
    Serialize,
};

use crate::{
    Rollbacks,
    SaveableRegistry,
    Snapshot,
    SnapshotDeserializer,
    SnapshotSerializer,
};

/// Extension trait that adds save-related methods to Bevy's `World`.
pub trait WorldSaveableExt: Sized {
    /// Returns a snapshot of the current World state.
    fn snapshot(&self) -> Snapshot;

    /// Creates a checkpoint for rollback.
    fn checkpoint(&mut self);

    /// Rolls back / forward the World state.
    fn rollback(&mut self, checkpoints: isize);

    /// Analogue of [`serde::Serialize`]
    ///
    /// # Errors
    /// See [`serde::Serialize`]
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;

    /// Analogue of [`serde::Deserialize`], but applies result to current `World` instead of creating a new one.
    ///
    /// # Errors
    /// See [`serde::Deserialize`]
    fn deserialize<'de, D: serde::Deserializer<'de>>(
        &mut self,
        deserializer: D,
    ) -> Result<(), D::Error>;

    /// Saves the game state to a named save.
    fn save(&self, name: &str);

    /// Loads the game state from a named save.
    fn load(&mut self, name: &str);
}

lazy_static! {
    static ref SAVE_DIR: PathBuf = {
        AppDirs::new(Some(env!("CARGO_PKG_NAME")), true)
            .unwrap()
            .data_dir
            .join("saves")
    };
}

/// Returns the absolute path to a save file given its name.
pub fn get_save_file(name: &str) -> PathBuf {
    SAVE_DIR.join(format!("{name}.sav"))
}

impl WorldSaveableExt for World {
    fn snapshot(&self) -> Snapshot {
        let registry = self.resource::<AppTypeRegistry>();
        let saveables = self.resource::<SaveableRegistry>();

        let mut resources = Vec::new();

        for reg in saveables.types() {
            if let Some(res) = reg.data::<ReflectResource>() {
                if let Some(reflect) = res.reflect(self) {
                    resources.push(reflect.clone_value());
                }
            }
        }

        let scene = DynamicScene::from_world(self, registry);

        Snapshot { resources, scene }
    }

    fn checkpoint(&mut self) {
        let snap = self.snapshot();

        let mut state = self.resource_mut::<Rollbacks>();

        state.checkpoint(snap);
    }

    fn rollback(&mut self, checkpoints: isize) {
        let mut state = self.resource_mut::<Rollbacks>();

        if let Some(snap) = state.rollback(checkpoints).cloned() {
            snap.apply(self);
        }
    }

    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let registry = self.resource::<AppTypeRegistry>();
        let snap = self.snapshot();

        // TODO: Include Rollbacks in the save Snapshot

        let ser = SnapshotSerializer::new(&snap, registry);

        ser.serialize(serializer)
    }

    fn deserialize<'de, D: serde::Deserializer<'de>>(
        &mut self,
        deserializer: D,
    ) -> Result<(), D::Error> {
        let registry = self.resource::<AppTypeRegistry>().clone();
        let reg = registry.read();
        
        let de = SnapshotDeserializer::new(&reg);

        let snap = de.deserialize(deserializer)?;

        snap.apply(self);

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
        let path = get_save_file(name);
        let file = File::open(path).expect("Could not open save file");

        let mut reader = BufReader::new(file);

        self.deserialize(&mut Deserializer::new(&mut reader))
            .expect("Error deserializing save from file");
    }
}
