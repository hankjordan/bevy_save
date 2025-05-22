//! Entity blueprints, with additional tools for saving and loading.

use bevy::prelude::*;

use crate::prelude::*;

#[allow(type_alias_bounds)]
/// [`QueryFilter`](bevy::ecs::query::QueryFilter) matching [`Prefab`].
pub type WithPrefab<P: Prefab> = With<P::Marker>;

/// Abstract spawning for entity types
pub trait Prefab: 'static {
    /// Marker component uniquely identifying the prefab entity
    ///
    /// This is automatically inserted for you when spawning the prefab.
    type Marker: Component + Default;

    /// Create a single instance of the prefab
    fn spawn(self, target: Entity, world: &mut World, target_original: Option<Entity>);

    /// Extract the prefab entities from the [`World`]
    fn extract(builder: SnapshotBuilder) -> SnapshotBuilder {
        builder.extract_entities_matching(|entity| entity.contains::<Self::Marker>())
    }
}
