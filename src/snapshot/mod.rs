//! Capturing and applying [`Snapshot`]s of application state.

mod applier;
mod builder;
mod snapshot;

pub use applier::{
    Hook,
    SnapshotApplier,
};
pub use builder::SnapshotBuilder;
pub use snapshot::Snapshot;
