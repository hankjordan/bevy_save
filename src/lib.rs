#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::module_inception)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::too_many_lines)]
#![doc = include_str!("../README.md")]

pub mod backend;
pub mod checkpoint;
mod clone;
pub mod commands;
pub mod dir;
mod error;
pub mod ext;
pub mod format;
pub mod middleware;
pub mod pipeline;
pub mod plugins;
pub mod serde;
pub mod snapshot;

pub use crate::{
    clone::CloneReflect,
    error::Error,
};

/// Prelude: convenient import for commonly used items provided by the crate.
#[allow(unused_imports)]
pub mod prelude {
    #[doc(inline)]
    pub use crate::{
        backend::{
            Backend,
            DefaultBackend,
            DefaultDebugBackend,
        },
        clone::CloneReflect,
        dir::{
            get_save_file,
            SAVE_DIR,
            WORKSPACE,
        },
        error::Error,
        ext::{
            AppCheckpointExt,
            AppSaveableExt,
            CommandsCheckpointExt,
            CommandsSaveableExt,
            WorldCheckpointExt,
            WorldSaveableExt,
        },
        format::{
            DefaultDebugFormat,
            DefaultFormat,
            Format,
        },
        middleware::*,
        pipeline::Pipeline,
        plugins::{
            SavePlugin,
            SavePlugins,
            SaveablesPlugin,
        },
        snapshot::{
            Hook,
            Snapshot,
            SnapshotApplier,
            SnapshotBuilder,
        },
    };
}
