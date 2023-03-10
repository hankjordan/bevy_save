use bevy::prelude::*;

use crate::RollbackSnapshot;

/// The global registry of snapshots used for rollback / rollforward.
#[derive(Resource, Clone, Default)]
pub struct Rollbacks {
    pub(crate) snapshots: Vec<RollbackSnapshot>,
    pub(crate) active: Option<usize>,
}

impl Rollbacks {
    /// Given a new [`RollbackSnapshot`], insert it and set it as the currently active rollback.
    ///
    /// If you rollback and then insert a checkpoint, it will erase all rollforward snapshots.
    pub fn checkpoint(&mut self, snapshot: RollbackSnapshot) {
        let active = self.active.unwrap_or(0);

        self.snapshots.truncate(active + 1);

        self.snapshots.push(snapshot);

        self.active = Some(self.snapshots.len() - 1);
    }

    /// Rolls back the given number of checkpoints.
    ///
    /// If checkpoints is negative, it rolls forward.
    ///
    /// This function will always clamp itself to valid snapshots.
    /// Rolling back or further farther than what is valid will just return the oldest / newest snapshot.
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_sign_loss)]
    pub fn rollback(&mut self, checkpoints: isize) -> Option<&RollbackSnapshot> {
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
