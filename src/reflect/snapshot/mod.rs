//! Capturing and applying [`Snapshot`]s of application state.

mod applier;
mod builder;
mod snapshot;

pub use self::{
    applier::{
        Applier,
        ApplierRef,
        BoxedHook,
        Hook,
    },
    builder::{
        Builder,
        BuilderRef,
    },
    snapshot::Snapshot,
};
