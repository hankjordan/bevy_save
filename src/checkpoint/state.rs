use bevy::{
    prelude::*,
    reflect::TypeRegistry,
};

use crate::prelude::*;

/// Currently stored snapshots used for rollback / rollforward.
#[derive(Resource, Default)]
pub struct Checkpoints {
    pub(crate) snapshots: Vec<Snapshot>,
    pub(crate) active: Option<usize>,
}

impl Checkpoints {
    /// Returns true if no checkpoints have been created.
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Given a new checkpoint [`Snapshot`], insert it and set it as the currently active checkpoint.
    ///
    /// If you rollback and then insert a checkpoint, it will erase all rollforward snapshots.
    pub fn checkpoint(&mut self, mut checkpoint: Snapshot) {
        let active = self.active.unwrap_or(0);

        // Force conversion into checkpoint
        checkpoint.checkpoints = None;

        self.snapshots.truncate(active + 1);
        self.snapshots.push(checkpoint);

        self.active = Some(self.snapshots.len() - 1);
    }

    /// Rolls back the given number of checkpoints.
    ///
    /// If checkpoints is negative, it rolls forward.
    ///
    /// This function will always clamp itself to valid rollbacks.
    /// Rolling back or further farther than what is valid will just return the oldest / newest snapshot.
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_sign_loss)]
    pub fn rollback(&mut self, checkpoints: isize) -> Option<&Snapshot> {
        if let Some(active) = self.active {
            let raw = active as isize - checkpoints;
            let new = raw.clamp(0, self.snapshots.len() as isize - 1) as usize;

            self.active = Some(new);
            Some(&self.snapshots[new])
        } else {
            None
        }
    }
}

impl CloneReflect for Checkpoints {
    fn clone_reflect(&self, registry: &TypeRegistry) -> Self {
        Self {
            snapshots: self.snapshots.clone_reflect(registry),
            active: self.active,
        }
    }
}
