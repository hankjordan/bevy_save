use std::collections::HashSet;

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
pub struct Snapshot {
    pub(crate) snapshot: RawSnapshot,
    pub(crate) rollbacks: Rollbacks,
}

impl Snapshot {
    pub(crate) fn default() -> Self {
        Self {
            snapshot: RawSnapshot::default(),
            rollbacks: Rollbacks::default(),
        }
    }
}

impl Snapshot {
    /// Returns a complete [`Snapshot`] of the current [`World`] state.
    ///
    /// Contains all saveable entities and resources, including [`Rollbacks`].
    ///
    /// # Shortcut for
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_save::prelude::*;
    /// # let world = &World::default();
    /// Snapshot::builder(world)
    ///     .extract_all()
    ///     .build();
    pub fn from_world(world: &World) -> Self {
        Self::builder(world).build()
    }

    /// Returns a [`Snapshot`] of the current [`World`] state filtered by `filter`.
    ///
    /// # Shortcut for
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_save::prelude::*;
    /// # let world = &World::default();
    /// # let filter = |_: &&bevy::reflect::TypeRegistration| true;
    /// Snapshot::builder(world)
    ///     .filter(filter)
    ///     .extract_all()
    ///     .build();
    /// ```
    pub fn from_world_with_filter<F>(world: &World, filter: F) -> Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        Self::builder(world).filter(filter).build()
    }

    /// Create a [`Builder`] from the [`World`], allowing you to create partial or filtered snapshots.
    pub fn builder(world: &World) -> Builder<Self> {
        Builder::new(world)
    }

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

impl<'w, F> Build for Builder<'w, Snapshot, F>
where
    F: Fn(&&TypeRegistration) -> bool,
{
    type Output = Snapshot;

    fn extract_entities(&mut self, entities: impl Iterator<Item = Entity>) -> &mut Self {
        let mut builder = Builder::new::<RawSnapshot>(self.world).filter(&self.filter);

        builder.extract_entities(entities);
        self.entities.append(&mut builder.entities);

        self
    }

    fn extract_all_entities(&mut self) -> &mut Self {
        self.extract_entities(self.world.iter_entities().map(|e| e.id()))
    }

    fn extract_resources<S: Into<String>>(
        &mut self,
        resources: impl Iterator<Item = S>,
    ) -> &mut Self {
        let resources = resources.map(|i| i.into()).collect::<HashSet<_>>();

        let mut builder = Builder::new::<RawSnapshot>(self.world).filter(&self.filter);

        builder.extract_resources(resources.iter());
        self.resources.append(&mut builder.resources);

        if resources.contains(std::any::type_name::<Rollbacks>()) {
            self.snapshot
                .get_or_insert_with(Snapshot::default)
                .rollbacks = self.world.resource::<Rollbacks>().clone_value();
        }

        self
    }

    fn extract_all_resources(&mut self) -> &mut Self {
        let mut builder = Builder::new::<RawSnapshot>(self.world).filter(&self.filter);

        builder.extract_all_resources();
        self.resources.append(&mut builder.resources);

        self.snapshot
            .get_or_insert_with(Snapshot::default)
            .rollbacks = self.world.resource::<Rollbacks>().clone_value();

        self
    }

    fn build(self) -> Self::Output {
        let mut snapshot = self.snapshot.unwrap_or_else(Snapshot::default);

        snapshot.snapshot = RawSnapshot {
            entities: self.entities.into_values().collect(),
            resources: self.resources.into_values().collect(),
        };

        snapshot
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
