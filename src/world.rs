use std::{
    fs,
    path::PathBuf,
};

use bevy::prelude::*;
use lazy_static::lazy_static;
use platform_dirs::AppDirs;

use crate::{
    Rollbacks,
    SaveableRegistry,
    Snapshot,
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
        info!("Directory {:?}", *SAVE_DIR);

        fs::create_dir_all(&*SAVE_DIR).expect("Could not create save directory");

        let registry = self.resource::<AppTypeRegistry>();

        let snap = self.snapshot();

        // Include Rollbacks in the save Snapshot

        todo!()
    }

    fn load(&mut self, name: &str) {
        todo!()
    }
}
