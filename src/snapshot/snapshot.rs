use bevy::{
    prelude::*,
    reflect::TypeRegistration,
};

use crate::{
    prelude::*,
    snapshot::RawSnapshot,
};

/// A complete snapshot of the game state.
///
/// Can be serialized via [`SnapshotSerializer`] and deserialized via [`SnapshotDeserializer`].
#[derive(Default)]
pub struct Snapshot {
    pub(crate) snapshot: RawSnapshot,
    pub(crate) rollbacks: Rollbacks,
}

impl Snapshot {
    /// Apply the [`Snapshot`] to the [`World`], restoring it to the saved state.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply(&self, world: &mut World) -> Result<(), SaveableError> {
        self.applier(world).apply()
    }

    /// Create an [`Applier`] from the [`Snapshot`] and the [`World`].
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy::ecs::entity::EntityMap;
    /// # use bevy_save::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = &mut app.world;
    /// let snapshot = Snapshot::from_world(world);
    ///
    /// snapshot
    ///     .applier(world)
    ///     .map(EntityMap::default())
    ///     .despawn(DespawnMode::default())
    ///     .mapping(MappingMode::default())
    ///     .apply();
    /// ```
    pub fn applier<'a>(&'a self, world: &'a mut World) -> Applier<&Self> {
        Applier::new(world, self)
    }

    /// Create an owning [`Applier`] from the [`Snapshot`] and the [`World`].
    pub fn into_applier(self, world: &mut World) -> Applier<Self> {
        Applier::new(world, self)
    }
}

impl Capture for Snapshot {
    fn extract_entities_with_filter<F>(
        &mut self,
        world: &World,
        entities: impl Iterator<Item = Entity>,
        filter: F,
    ) -> &mut Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        self.snapshot
            .extract_entities_with_filter(world, entities, filter);
        self
    }

    fn extract_resources_with_filter<F>(&mut self, world: &World, filter: F) -> &mut Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        self.snapshot.extract_resources_with_filter(world, filter);
        self.rollbacks = world.resource::<Rollbacks>().clone_value();
        self
    }

    fn clear(&mut self) -> &mut Self {
        self.clear_entities().clear_resources()
    }

    fn clear_entities(&mut self) -> &mut Self {
        self.snapshot.clear_entities();
        self
    }

    fn clear_resources(&mut self) -> &mut Self {
        self.snapshot.clear_resources();
        self.rollbacks = Rollbacks::default();
        self
    }

    fn remove_empty(&mut self) -> &mut Self {
        self.snapshot.remove_empty();
        self
    }
}

macro_rules! impl_snapshot_applier {
    ($t:ty) => {
        impl<'a> Applier<'a, $t> {
            /// Apply the [`Snapshot`].
            ///
            /// # Errors
            /// - See [`SaveableError`]
            pub fn apply(self) -> Result<(), SaveableError> {
                let applier = Applier {
                    world: self.world,
                    snapshot: &self.snapshot.snapshot,
                    map: self.map,
                    despawn: self.despawn,
                    mapping: self.mapping,
                    hook: self.hook,
                };

                applier.apply()?;

                self.world
                    .insert_resource(self.snapshot.rollbacks.clone_value());

                Ok(())
            }
        }
    };
}

impl_snapshot_applier!(Snapshot);
impl_snapshot_applier!(&'a Snapshot);

impl CloneReflect for Snapshot {
    fn clone_value(&self) -> Self {
        Self {
            snapshot: self.snapshot.clone_value(),
            rollbacks: self.rollbacks.clone_value(),
        }
    }
}
