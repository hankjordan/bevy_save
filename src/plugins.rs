use bevy::{
    app::PluginGroupBuilder,
    prelude::*,
};

use crate::prelude::*;

/// Default plugins for `bevy_save`.
pub struct SavePlugins;

impl PluginGroup for SavePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(SavePlugin)
            .add(SaveablesPlugin)
    }
}

/// `bevy_save` core functionality plugin.
pub struct SavePlugin;

#[rustfmt::skip]
impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SaveableRegistry>()
            .init_resource::<Rollbacks>();
    }
}

/// Saveable registrations for common types.
pub struct SaveablesPlugin;

#[rustfmt::skip]
impl Plugin for SaveablesPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_saveable::<GlobalTransform>()
            .register_saveable::<Transform>();

        #[cfg(feature = "bevy_render")]
        app
            .register_saveable::<ComputedVisibility>()
            .register_saveable::<Handle<Image>>()
            .register_saveable::<Visibility>();

        #[cfg(feature = "bevy_sprite")]
        app
            .register_saveable::<Sprite>()

            // Fix `bevy_reflect: Add ReflectComponent registration for Sprite #8206`
            .register_type_data::<Sprite, ReflectComponent>()
            .register_type::<Option<Vec2>>()
            .register_type::<Option<Rect>>();
    }
}
