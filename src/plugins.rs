use bevy::{
    app::PluginGroupBuilder,
    prelude::*,
};

use crate::{
    dynamic::RollbackRegistry,
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

#[rustfmt::skip]
impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_pipeline::<&str>()
            .init_pipeline::<DebugPipeline>()
            
            .init_resource::<RollbackRegistry>()
            .init_resource::<Rollbacks>();
    }
}

/// Saveable registrations for common types.
pub struct SaveablesPlugin;

#[rustfmt::skip]
impl Plugin for SaveablesPlugin {
    fn build(&self, app: &mut App) {
        
        #[cfg(feature = "bevy_render")]
        app
            .register_type::<Color>();

        #[cfg(feature = "bevy_sprite")]
        app
            .register_type::<Option<Vec2>>()
            .register_type::<Option<Rect>>();
    }
}
