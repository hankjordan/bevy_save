//! Checkpoint utilities for [`Snapshot`](crate::prelude::Snapshot)s that can be quickly rolled through.

mod ext;
mod registry;
mod state;

pub use self::{
    ext::{
        AppCheckpointExt,
        WorldCheckpointExt,
    },
    registry::CheckpointRegistry,
    state::Checkpoints,
};
