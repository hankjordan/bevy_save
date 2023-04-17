use bevy::prelude::*;

use crate::prelude::*;

/// The global registry of snapshots used for rollback / rollforward.
#[derive(Resource, Default)]
pub struct Rollbacks {
    pub(crate) rollbacks: Vec<Rollback>,
    pub(crate) active: Option<usize>,
}

impl Rollbacks {
    /// Given a new [`Rollback`], insert it and set it as the currently active rollback.
    ///
    /// If you rollback and then insert a checkpoint, it will erase all rollforward snapshots.
    pub fn checkpoint(&mut self, rollback: Rollback) {
        let active = self.active.unwrap_or(0);

        self.rollbacks.truncate(active + 1);
        self.rollbacks.push(rollback);

        self.active = Some(self.rollbacks.len() - 1);
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
            let new = raw.clamp(0, self.rollbacks.len() as isize - 1) as usize;

            self.active = Some(new);
            Some(&self.rollbacks[new])
        } else {
            None
        }
    }
}

impl CloneReflect for Rollbacks {
    fn clone_value(&self) -> Self {
        Self {
            rollbacks: self.rollbacks.iter().map(|r| r.clone_value()).collect(),
            active: self.active,
        }
    }
}
