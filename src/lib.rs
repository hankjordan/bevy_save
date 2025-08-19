// `rustdoc_internals` is needed for `#[doc(fake_variadics)]`
#![allow(unexpected_cfgs)]
#![allow(internal_features)]
#![cfg_attr(any(docsrs, docsrs_dep), feature(doc_auto_cfg, rustdoc_internals))]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::module_inception)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::too_many_lines)]
#![doc = include_str!("../README.md")]

pub mod backend;
pub mod dir;
mod error;
pub mod flows;
pub mod format;
pub mod middleware;
pub mod plugins;
mod utils;

/// `bevy_save` crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "reflect")]
/// `bevy_save` snapshot version
pub const SNAPSHOT_VERSION: reflect::SnapshotVersion = reflect::SnapshotVersion::V4;

#[cfg(feature = "reflect")]
pub mod reflect;

#[cfg(feature = "reflect")]
pub use crate::reflect::clone_reflect_value;
pub use crate::{
    error::Error,
    utils::{
        IntoVersion,
        MaybeMut,
        MaybeRef,
    },
};

/// Prelude: convenient import for commonly used items provided by the crate.
#[allow(unused_imports)]
pub mod prelude {
    pub use bevy_save_macros::FlowLabel;

    #[cfg(all(feature = "reflect", feature = "checkpoints"))]
    #[doc(inline)]
    pub use crate::reflect::checkpoint::{
        ReflectIgnoreCheckpoint,
        WorldCheckpointExt,
    };
    #[cfg(feature = "reflect")]
    #[doc(inline)]
    pub use crate::reflect::{
        Pipeline,
        ReflectIgnore,
        migration::{
            Migrate,
            Migrator,
            ReflectMigrate,
        },
        pipeline::AppPipelineExt,
        prefab::{
            CommandsPrefabExt,
            Prefab,
            WithPrefab,
        },
        relationship::{
            ReflectRelationship,
            ReflectRelationshipTarget,
        },
        snapshot::{
            Applier,
            ApplierRef,
            BoxedHook,
            Builder,
            BuilderRef,
            Hook,
            Snapshot,
        },
    };
    #[doc(inline)]
    pub use crate::{
        backend::{
            AppBackendExt,
            Backend,
            DefaultBackend,
            DefaultDebugBackend,
        },
        dir::{
            SAVE_DIR,
            WORKSPACE,
            get_save_file,
        },
        error::Error,
        flows::{
            AppFlowExt,
            Flow,
            FlowLabel,
            FlowSystem,
            Flows,
            IntoFlowSystems,
            pathway::{
                AppPathwayExt,
                CaptureDeserialize,
                CaptureInput,
                CaptureOutput,
                CaptureSerialize,
                Pathway,
                WorldPathwayExt,
            },
        },
        format::{
            DefaultDebugFormat,
            DefaultFormat,
            Format,
        },
        middleware::*,
        plugins::{
            SaveCorePlugin,
            SavePlugins,
        },
    };
}
