//! Capturing and applying [`Snapshot`]s of application state.

mod applier;
mod builder;
mod snapshot;

pub use self::{
    applier::{
        BoxedHook,
        Hook,
        SnapshotApplier,
    },
    builder::SnapshotBuilder,
    snapshot::Snapshot,
};
