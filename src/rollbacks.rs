use bevy::prelude::*;

use crate::prelude::*;

/// The global registry of snapshots used for rollback / rollforward.
#[derive(Resource, Default)]
pub struct Rollbacks {
    pub(crate) checkpoints: Vec<Rollback>,
    pub(crate) active: Option<usize>,
}

impl Rollbacks {
    /// Returns true if no checkpoints have been created.
    pub fn is_empty(&self) -> bool {
        self.checkpoints.is_empty()
    }

    /// Given a new [`Rollback`], insert it and set it as the currently active rollback.
    ///
    /// If you rollback and then insert a checkpoint, it will erase all rollforward snapshots.
    pub fn checkpoint(&mut self, rollback: Rollback) {
        let active = self.active.unwrap_or(0);

        self.checkpoints.truncate(active + 1);
        self.checkpoints.push(rollback);

        self.active = Some(self.checkpoints.len() - 1);
    }

    /// Rolls back the given number of checkpoints.
    ///
    /// If checkpoints is negative, it rolls forward.
    ///
    /// This function will always clamp itself to valid rollbacks.
    /// Rolling back or further farther than what is valid will just return the oldest / newest snapshot.
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_sign_loss)]
    pub fn rollback(&mut self, checkpoints: isize) -> Option<&Rollback> {
        if let Some(active) = self.active {
            let raw = active as isize - checkpoints;
            let new = raw.clamp(0, self.checkpoints.len() as isize - 1) as usize;

            self.active = Some(new);
            Some(&self.checkpoints[new])
        } else {
            None
        }
    }
}

impl CloneReflect for Rollbacks {
    fn clone_value(&self) -> Self {
        Self {
            checkpoints: self.checkpoints.iter().map(|r| r.clone_value()).collect(),
            active: self.active,
        }
    }
}
