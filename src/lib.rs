#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(private_bounds)]
#![allow(private_interfaces)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::module_inception)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::too_many_lines)]
#![doc = include_str!("../README.md")]

mod app;
mod backend;
mod error;
mod format;
mod hook;
mod middleware;
mod pipeline;
mod plugins;
mod world;

/// Save directory management, workspace information
pub mod dir;
/// Reflection-based snapshots and rollbacks
pub mod dynamic;
/// Statically typed snapshots
pub mod typed;

pub use crate::{
    error::Error,
    hook::Hook,
};

/// Prelude: convenient import for all the user-facing APIs provided by the crate
pub mod prelude {
    pub use crate::{
        app::AppSaveableExt,
        backend::{
            Backend,
            DefaultBackend,
            DefaultDebugBackend,
        },
        dynamic::{
            DynamicSnapshot,
            DynamicSnapshotApplier,
            DynamicSnapshotBuilder,
            Rollbacks,
        },
        format::{
            DefaultDebugFormat,
            DefaultFormat,
            Format,
        },
        pipeline::{
            DebugPipeline,
            DynamicPipeline,
        },
        plugins::{
            SavePlugin,
            SavePlugins,
            SaveablesPlugin,
        },
        typed::{
            Snapshot,
            SnapshotApplier,
            SnapshotBuilder,
        },
        world::{
            WorldRollbackExt,
            WorldSaveableExt,
        },
    };
}
