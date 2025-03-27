//! Extension traits.

mod app;
mod commands;
mod world;

pub use self::{
    app::{
        AppCheckpointExt,
        AppSaveableExt,
    },
    commands::{
        CommandsCheckpointExt,
        CommandsPrefabExt,
        CommandsSaveableExt,
    },
    world::{
        WorldCheckpointExt,
        WorldSaveableExt,
    },
};
