//! Bevy plugins necessary for the crate to function.

use bevy::{
    app::PluginGroupBuilder,
    prelude::*,
};

use crate::prelude::*;

/// Default plugins for `bevy_save`.
pub struct SavePlugins;

impl PluginGroup for SavePlugins {
    fn build(self) -> PluginGroupBuilder {
        let b = PluginGroupBuilder::start::<Self>().add(SavePlugin);

        #[cfg(feature = "checkpoints")]
        let b = b.add(SaveCheckpointsPlugin);

        #[cfg(feature = "reflect")]
        let b = b.add(SaveReflectPlugin);

        b
    }
}

/// `bevy_save` core functionality.
///
/// If you don't wish to use reflection, this is all you will need.
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_backend::<DefaultBackend, String>()
            .init_backend::<DefaultBackend, &str>()
            .init_backend::<DefaultDebugBackend, String>()
            .init_backend::<DefaultDebugBackend, &str>();
    }
}

/// `bevy_save` checkpoint functionality.
#[cfg(feature = "checkpoints")]
pub struct SaveCheckpointsPlugin;

#[cfg(feature = "checkpoints")]
impl Plugin for SaveCheckpointsPlugin {
    fn build(&self, app: &mut App) {
        use crate::reflect::{
            backcompat::v0_16::CheckpointsV0_16,
            checkpoint::{
                CheckpointRegistry,
                Checkpoints,
            },
        };

        app.register_type::<Checkpoints>()
            .register_type::<CheckpointsV0_16>()
            .init_resource::<CheckpointRegistry>()
            .init_resource::<Checkpoints>();
    }
}

/// Type registrations for reflect types.
#[cfg(feature = "reflect")]
pub struct SaveReflectPlugin;

#[cfg(feature = "reflect")]
impl Plugin for SaveReflectPlugin {
    fn build(&self, app: &mut App) {
        use crate::reflect::backcompat::v0_16::SnapshotV0_16;

        app.register_type::<Snapshot>()
            .register_type::<SnapshotV0_16>();

        #[cfg(feature = "bevy_render")]
        app.register_type::<Color>();

        #[cfg(feature = "bevy_sprite")]
        app.register_type::<Option<Vec2>>()
            .register_type::<Option<Rect>>();
    }
}
