use std::{
    fs::{
        self,
        File,
    },
    io::Write,
    path::PathBuf,
};

use bevy::{
    prelude::*,
    tasks::IoTaskPool,
};
use lazy_static::lazy_static;
use platform_dirs::AppDirs;
use rmp_serde::Serializer;
use serde::Serialize;

use crate::{
    Rollbacks,
    SaveableRegistry,
    Snapshot,
    SnapshotSerializer,
};

/// Extension trait that adds save-related methods to Bevy's `World`.
pub trait WorldSaveableExt {
    /// Returns a snapshot of the current World state.
    fn snapshot(&self) -> Snapshot;

    /// Creates a checkpoint for rollback.
    fn checkpoint(&mut self);

    /// Rolls back / forward the World state.
    fn rollback(&mut self, checkpoints: isize);

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

    fn save(&self, name: &str) {
        fs::create_dir_all(&*SAVE_DIR).expect("Could not create save directory");

        let registry = self.resource::<AppTypeRegistry>();

        let snap = self.snapshot();

        // TODO: Include Rollbacks in the save Snapshot

        let ser = SnapshotSerializer::new(&snap, registry);

        let mut buf = Vec::new();

        ser.serialize(&mut Serializer::new(&mut buf))
            .expect("Error serializing Snapshot");

        let name = name.to_owned();

        IoTaskPool::get()
            .spawn(async move {
                File::create(SAVE_DIR.join(format!("{name}.sav")))
                    .and_then(|mut file| file.write(buf.as_slice()))
                    .expect("Error writing Snapshot to file");
            })
            .detach();
    }

    fn load(&mut self, name: &str) {
        todo!()
    }
}
