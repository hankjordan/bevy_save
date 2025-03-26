//! Extension traits.

mod app;
mod commands;
mod world;

pub use app::{
    AppCheckpointExt,
    AppSaveableExt,
};
pub use commands::{
    CommandsCheckpointExt,
    CommandsSaveableExt,
};
pub use world::{
    WorldCheckpointExt,
    WorldSaveableExt,
};
