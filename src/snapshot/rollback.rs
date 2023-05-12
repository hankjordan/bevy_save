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
#[derive(Default)]
pub struct Rollback {
    pub(crate) snapshot: RawSnapshot,
}

impl Rollback {
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

impl Capture for Rollback {
    fn extract_entities_with_filter<F>(
        &mut self,
        world: &World,
        entities: impl Iterator<Item = Entity>,
        filter: F,
    ) -> &mut Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        let registry = world.resource::<SaveableRegistry>();

        self.snapshot
            .extract_entities_with_filter(world, entities, |reg| {
                registry.can_rollback(reg.type_name()) && filter(reg)
            });

        self
    }

    fn extract_resources_with_filter<F>(&mut self, world: &World, filter: F) -> &mut Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        let registry = world.resource::<SaveableRegistry>();

        self.snapshot.extract_resources_with_filter(world, |reg| {
            registry.can_rollback(reg.type_name()) && filter(reg)
        });

        self
    }

    fn clear(&mut self) -> &mut Self {
        self.snapshot.clear();
        self
    }

    fn clear_entities(&mut self) -> &mut Self {
        self.snapshot.clear_entities();
        self
    }

    fn clear_resources(&mut self) -> &mut Self {
        self.snapshot.clear_resources();
        self
    }

    fn remove_empty(&mut self) -> &mut Self {
        self.snapshot.remove_empty();
        self
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
