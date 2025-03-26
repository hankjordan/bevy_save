//! Checkpoint utilities for [`Snapshot`](crate::prelude::Snapshot)s that can be quickly rolled through.

mod registry;
mod state;

pub use registry::CheckpointRegistry;
pub use state::Checkpoints;
