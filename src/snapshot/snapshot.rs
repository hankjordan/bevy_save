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
