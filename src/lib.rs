#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![doc = include_str!("../README.md")]

pub use crate::{
    app::*,
    reflect::*,
    registry::*,
    rollbacks::*,
    serde::*,
    snapshot::*,
    world::*,
};

mod app;
mod reflect;
mod registry;
mod rollbacks;
mod scene;
mod serde;
mod snapshot;
mod world;

/// Prelude: convenient import for all the user-facing APIs provided by the crate
pub mod prelude {
    pub use crate::*;
}
