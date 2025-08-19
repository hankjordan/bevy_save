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
        let b = PluginGroupBuilder::start::<Self>().add(SaveCorePlugin);

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
pub struct SaveCorePlugin;

impl Plugin for SaveCorePlugin {
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
            checkpoint::Checkpoints,
            migration::backcompat::v3::CheckpointsV3,
        };

        app.register_type::<Checkpoints>()
            .register_type::<CheckpointsV3>()
            .init_resource::<Checkpoints>();
    }
}

/// Type registrations for reflect types.
#[cfg(feature = "reflect")]
pub struct SaveReflectPlugin;

#[cfg(feature = "reflect")]
impl Plugin for SaveReflectPlugin {
    fn build(&self, app: &mut App) {
        use crate::reflect::migration::backcompat::v3::SnapshotV3;

        app.register_type::<Snapshot>()
            .register_type::<SnapshotV3>();

        app.register_type_data::<ChildOf, ReflectRelationship>()
            .register_type_data::<Children, ReflectRelationshipTarget>();

        #[cfg(feature = "bevy_render")]
        app.register_type::<Color>();

        #[cfg(feature = "bevy_sprite")]
        app.register_type::<Option<Vec2>>()
            .register_type::<Option<Rect>>();

        #[cfg(feature = "log")]
        app.add_systems(PostStartup, |registry: Res<AppTypeRegistry>| {
            const REGISTERED: &str = "registered for";
            const WITHOUT: &str = "without registering";
            const FOR: &str = "for target";
            const MISSING: &str = "are you missing a";
            const UNKNOWN: &str = "<unregistered type>";

            let registry = registry.read();

            for (ty, data) in registry.iter_with_data::<ReflectRelationship>() {
                let t_id = data.target();
                let t_ty = registry.get(t_id);
                let t_type_path = t_ty.map(|r| r.type_info().type_path());

                if t_ty
                    .filter(|ty| ty.contains::<ReflectRelationshipTarget>())
                    .is_none()
                {
                    bevy::log::warn!(
                        "`{}` {REGISTERED} {:?} {WITHOUT} `{}` {FOR} {:?} ({MISSING} `#[reflect({})]`?)",
                        "ReflectRelationship",
                        ty.type_info().type_path(),
                        "ReflectRelationshipTarget",
                        t_type_path.unwrap_or(UNKNOWN),
                        "RelationshipTarget",
                    );
                }
            }

            for (ty, data) in registry.iter_with_data::<ReflectRelationshipTarget>() {
                let r_id = data.relationship();
                let r_ty = registry.get(r_id);
                let r_type_path = r_ty.map(|r| r.type_info().type_path());

                if r_ty
                    .filter(|ty| ty.contains::<ReflectRelationship>())
                    .is_none()
                {
                    bevy::log::warn!(
                        "`{}` {REGISTERED} {:?} {WITHOUT} `{}` {FOR} {:?} ({MISSING} `#[reflect({})]`?)",
                        "ReflectRelationshipTarget",
                        ty.type_info().type_path(),
                        "ReflectRelationship",
                        r_type_path.unwrap_or(UNKNOWN),
                        "Relationship",
                    );
                }
            }
        });
    }
}
