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
        let b = b.add(SaveablesPlugin);

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
        use crate::reflect::checkpoint::{
            CheckpointRegistry,
            Checkpoints,
        };

        app.init_resource::<CheckpointRegistry>()
            .init_resource::<Checkpoints>();
    }
}

/// Type registrations for common types.
pub struct SaveablesPlugin;

impl Plugin for SaveablesPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "bevy_render")]
        app.register_type::<Color>();

        #[cfg(feature = "bevy_sprite")]
        app.register_type::<Option<Vec2>>()
            .register_type::<Option<Rect>>();
    }
}
