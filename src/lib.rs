#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::module_inception)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::too_many_lines)]
#![doc = include_str!("../README.md")]

pub use bevy_save_erased_serde as erased_serde;

pub use crate::{
    app::*,
    applier::*,
    clone::*,
    dir::*,
    error::*,
    plugins::*,
    registry::*,
    rollbacks::*,
    saver::*,
    serde::*,
    snapshot::*,
    world::*,
};

mod app;
mod applier;
mod clone;
mod dir;
mod entity;
mod error;
mod plugins;
mod registry;
mod rollbacks;
mod saver;
mod serde;
mod snapshot;
mod world;

/// Prelude: convenient import for all the user-facing APIs provided by the crate
pub mod prelude {
    pub use crate::{
        app::*,
        applier::*,
        clone::*,
        dir::*,
        erased_serde::{
            IntoDeserializer,
            IntoSerializer,
        },
        error::*,
        plugins::*,
        registry::*,
        rollbacks::*,
        saver::*,
        serde::*,
        snapshot::*,
        world::*,
    };
}
