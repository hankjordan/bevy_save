use bevy::{
    prelude::*,
    reflect::TypeRegistration,
};

use crate::{
    prelude::*,
    snapshot::RawSnapshot,
};

/// A rollback snapshot of the game state.
///
/// [`Rollback`] excludes types that opt out of rollback.
pub struct Rollback {
    pub(crate) snapshot: RawSnapshot,
}

impl Rollback {
    /// Returns a [`Rollback`] of the current [`World`] state.
    ///
    /// This excludes [`Rollbacks`] and any saveable that ignores rollbacking.
    ///
    /// # Shortcut for
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_save::prelude::*;
    /// # let world = &World::default();
    /// Rollback::builder(world)
    ///     .build();
    /// ```
    pub fn from_world(world: &World) -> Self {
        Self::builder(world).build()
    }

    /// Returns a [`Rollback`] of the current [`World`] state filtered by `filter`.
    ///
    /// # Shortcut for
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_save::prelude::*;
    /// # let world = &World::default();
    /// # let filter = |_: &&bevy::reflect::TypeRegistration| true;
    /// Rollback::builder(world)
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

    /// Apply the [`Rollback`] to the [`World`].
    ///
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply(&self, world: &mut World) -> Result<(), SaveableError> {
        self.applier(world).apply()
    }

    /// Create an [`Applier`] from the [`Rollback`] and the [`World`].
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy::ecs::entity::EntityMap;
    /// # use bevy_save::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = &mut app.world;
    /// let rollback = Rollback::from_world(world);
    ///
    /// rollback
    ///     .applier(world)
    ///     .map(EntityMap::default())
    ///     .despawn(DespawnMode::default())
    ///     .mapping(MappingMode::default())
    ///     .apply();
    /// ```
    pub fn applier<'a>(&'a self, world: &'a mut World) -> Applier<'a, &'a Self> {
        Applier::new(world, self)
    }

    /// Create an owning [`Applier`] from the [`Rollback`] and the [`World`].
    pub fn into_applier(self, world: &mut World) -> Applier<Self> {
        Applier::new(world, self)
    }
}

impl<'w, F> Build for Builder<'w, Rollback, F>
where
    F: Fn(&&TypeRegistration) -> bool,
{
    type Output = Rollback;

    fn extract_entities(&mut self, entities: impl Iterator<Item = Entity>) -> &mut Self {
        let registry = self.world.resource::<SaveableRegistry>();

        let mut builder =
            Builder::new::<RawSnapshot>(self.world).filter(|reg: &&TypeRegistration| {
                registry.can_rollback(reg.type_name()) && (self.filter)(reg)
            });

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
        let registry = self.world.resource::<SaveableRegistry>();

        let mut builder =
            Builder::new::<RawSnapshot>(self.world).filter(|reg: &&TypeRegistration| {
                registry.can_rollback(reg.type_name()) && (self.filter)(reg)
            });

        builder.extract_resources(resources);
        self.resources.append(&mut builder.resources);

        self
    }

    fn extract_all_resources(&mut self) -> &mut Self {
        let registry = self.world.resource::<SaveableRegistry>();

        let mut builder =
            Builder::new::<RawSnapshot>(self.world).filter(|reg: &&TypeRegistration| {
                registry.can_rollback(reg.type_name()) && (self.filter)(reg)
            });

        builder.extract_all_resources();
        self.resources.append(&mut builder.resources);

        self
    }

    fn build(self) -> Self::Output {
        Rollback {
            snapshot: RawSnapshot {
                entities: self.entities.into_values().collect(),
                resources: self.resources.into_values().collect(),
            },
        }
    }
}

macro_rules! impl_rollback_applier {
    ($t:ty) => {
        impl<'a> Applier<'a, $t> {
            /// Apply the [`Rollback`].
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

                applier.apply()
            }
        }
    };
}

impl_rollback_applier!(Rollback);
impl_rollback_applier!(&'a Rollback);

impl CloneReflect for Rollback {
    fn clone_value(&self) -> Self {
        Self {
            snapshot: self.snapshot.clone_value(),
        }
    }
}
