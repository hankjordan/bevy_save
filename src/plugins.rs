//! Bevy plugins necessary for the crate to function.

use bevy::{
    app::PluginGroupBuilder,
    prelude::*,
};

use crate::{
    checkpoint::{
        CheckpointRegistry,
        Checkpoints,
    },
    prelude::*,
};

/// Default plugins for `bevy_save`.
pub struct SavePlugins;

impl PluginGroup for SavePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(SavePlugin)
            .add(SaveablesPlugin)
    }
}

/// `bevy_save` core functionality.
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DefaultBackend>()
            .init_resource::<DefaultDebugBackend>()
            .init_resource::<CheckpointRegistry>()
            .init_resource::<Checkpoints>();
    }
}

/// Saveable registrations for common types.
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
