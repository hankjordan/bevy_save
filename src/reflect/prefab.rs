//! Entity blueprints, with additional tools for saving and loading.

use bevy::prelude::*;

use crate::prelude::*;

/// [`QueryFilter`](bevy::ecs::query::QueryFilter) matching [`Prefab`].
pub type WithPrefab<P> = With<<P as Prefab>::Marker>;

/// Abstract spawning for entity types
pub trait Prefab: 'static {
    /// Marker component uniquely identifying the prefab entity
    ///
    /// This is automatically inserted for you when spawning the prefab.
    type Marker: Component + Default;

    /// Create a single instance of the prefab
    fn spawn(self, target: Entity, world: &mut World);

    /// Extract the prefab entities from the [`World`]
    fn extract(builder: BuilderRef) -> BuilderRef {
        builder.extract_entities_matching(|entity| entity.contains::<Self::Marker>())
    }
}

/// Spawn an instance of the [`Prefab`].
pub struct SpawnPrefabCommand<P> {
    target: Entity,
    prefab: P,
}

impl<P> SpawnPrefabCommand<P> {
    /// Create a [`SpawnPrefabCommand`] from the target entity and [`Prefab`].
    pub fn new(target: Entity, prefab: P) -> Self {
        Self { target, prefab }
    }
}

impl<P: Prefab + Send + 'static> Command for SpawnPrefabCommand<P> {
    fn apply(self, world: &mut World) {
        self.prefab.spawn(self.target, world);
    }
}

/// Extension trait that adds prefab-related methods to Bevy's [`Commands`].
pub trait CommandsPrefabExt {
    /// Spawn a [`Prefab`] entity.
    fn spawn_prefab<P: Prefab + Send + 'static>(&mut self, prefab: P) -> EntityCommands;
}

impl CommandsPrefabExt for Commands<'_, '_> {
    fn spawn_prefab<P: Prefab + Send + 'static>(&mut self, prefab: P) -> EntityCommands {
        let target = self.spawn(P::Marker::default()).id();
        self.queue(SpawnPrefabCommand::new(target, prefab));
        self.entity(target)
    }
}
