use bevy::ecs::{
    system::EntityCommands,
    world::EntityRef,
};

/// A [`Hook`] runs on each entity when applying a snapshot.
///
/// # Example
/// This could be used to apply entities as children of another entity.
/// ```
/// # use bevy::prelude::*;
/// # use bevy_save::prelude::*;
/// # let mut app = App::new();
/// # app.add_plugins(MinimalPlugins);
/// # app.add_plugins(SavePlugins);
/// # let world = &mut app.world;
/// # let snapshot = DynamicSnapshot::from_world(world);
/// # let parent = world.spawn_empty().id();
/// snapshot
///     .applier(world)
///     .hook(move |entity, cmds| {
///         if !entity.contains::<Parent>() {
///             cmds.set_parent(parent);
///         }
///     })
///     .apply();
/// ```
pub trait Hook: for<'a> Fn(&'a EntityRef, &'a mut EntityCommands) + Send + Sync {}

impl<T> Hook for T where T: for<'a> Fn(&'a EntityRef, &'a mut EntityCommands) + Send + Sync {}
