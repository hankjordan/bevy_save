#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![doc = include_str!("../README.md")]

pub use crate::{
    app::*,
    clone::*,
    dir::*,
    entity::*,
    error::*,
    plugins::*,
    registry::*,
    rollbacks::*,
    serde::*,
    snapshot::*,
    world::*,
};

mod app;
mod clone;
mod dir;
mod entity;
mod error;
mod plugins;
mod registry;
mod rollbacks;
mod serde;
mod snapshot;
mod world;

/// Prelude: convenient import for all the user-facing APIs provided by the crate
pub mod prelude {
    pub use crate::*;
}
