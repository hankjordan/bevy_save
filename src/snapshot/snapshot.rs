use std::collections::HashSet;

use bevy::{
    prelude::*,
    reflect::TypeRegistration,
};

use crate::{
    prelude::*,
    Error,
    snapshot::RawSnapshot,
};

/// A complete snapshot of the game state.
///
/// Can be serialized via [`SnapshotSerializer`] and deserialized via [`SnapshotDeserializer`].
#[derive(Debug)]
pub struct Snapshot {
    pub(crate) snapshot: RawSnapshot,
    pub(crate) rollbacks: Option<Rollbacks>,
}

impl Snapshot {
    pub(crate) fn default() -> Self {
        Self {
            snapshot: RawSnapshot::default(),
            rollbacks: None,
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
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = &mut app.world;
    /// Snapshot::builder(world)
    ///     .extract_all()
    ///     .build();
    pub fn from_world(world: &World) -> Self {
        Self::builder(world).extract_all().build()
    }

    /// Returns a [`Snapshot`] of the current [`World`] state filtered by `filter`.
    ///
    /// # Shortcut for
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_save::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = &mut app.world;
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
        Self::builder(world).filter(filter).extract_all().build()
    }

    /// Create a [`Builder`] from the [`World`], allowing you to create partial or filtered snapshots.
    /// 
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_save::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = &mut app.world;
    /// Snapshot::builder(world)
    ///     // Exclude `Transform` from this `Snapshot`
    ///     .filter(|reg| reg.type_info().type_path() != "bevy_transform::components::transform::Transform")
    ///
    ///     // Extract all matching entities and resources
    ///     .extract_all()
    ///
    ///     // Clear all extracted entities without any components
    ///     .clear_empty()
    ///
    ///     // Build the `Snapshot`
    ///     .build();
    /// ```
    pub fn builder(world: &World) -> Builder<Self> {
        Builder::new(world)
    }

    /// Apply the [`Snapshot`] to the [`World`], restoring it to the saved state.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply(&self, world: &mut World) -> Result<(), Error> {
        self.applier(world).apply()
    }

    /// Create an [`Applier`] from the [`Snapshot`] and the [`World`].
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy::utils::HashMap;
    /// # use bevy_save::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = &mut app.world;
    /// let snapshot = Snapshot::from_world(world);
    ///
    /// snapshot
    ///     .applier(world)
    ///     .map(HashMap::default())
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

    fn extract_entities(mut self, entities: impl Iterator<Item = Entity>) -> Self {
        let mut builder = Builder::new::<RawSnapshot>(self.world)
            .filter(&self.filter)
            .extract_entities(entities);

        self.entities.append(&mut builder.entities);

        self
    }

    fn extract_all_entities(self) -> Self {
        let world = self.world;
        self.extract_entities(world.iter_entities().map(|e| e.id()))
    }

    fn extract_resources<S: Into<String>>(mut self, resources: impl Iterator<Item = S>) -> Self {
        let resources = resources.map(|i| i.into()).collect::<HashSet<_>>();

        let mut builder = Builder::new::<RawSnapshot>(self.world)
            .filter(&self.filter)
            .extract_resources(resources.iter());

        self.resources.append(&mut builder.resources);

        if resources.contains(std::any::type_name::<Rollbacks>()) {
            if let Some(rollbacks) = self.world.get_resource::<Rollbacks>() {
                if !rollbacks.is_empty() {
                    self.snapshot
                        .get_or_insert_with(Snapshot::default)
                        .rollbacks = Some(rollbacks.clone_value());
                }
            }
        }

        self
    }

    fn extract_all_resources(mut self) -> Self {
        let mut builder = Builder::new::<RawSnapshot>(self.world)
            .filter(&self.filter)
            .extract_all_resources();

        self.resources.append(&mut builder.resources);

        if let Some(rollbacks) = self.world.get_resource::<Rollbacks>() {
            if !rollbacks.is_empty() {
                self.snapshot
                    .get_or_insert_with(Snapshot::default)
                    .rollbacks = Some(rollbacks.clone_value());
            }
        }

        self
    }

    fn clear_entities(mut self) -> Self {
        self.entities.clear();
        self
    }

    fn clear_resources(mut self) -> Self {
        self.resources.clear();

        if let Some(snapshot) = &mut self.snapshot {
            snapshot.rollbacks = None;
        }

        self
    }

    fn clear_empty(mut self) -> Self {
        self.entities.retain(|_, e| !e.is_empty());
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
            pub fn apply(self) -> Result<(), Error> {
                let applier = Applier {
                    world: self.world,
                    snapshot: &self.snapshot.snapshot,
                    map: self.map,
                    despawn: self.despawn,
                    mapping: self.mapping,
                    hook: self.hook,
                };

                applier.apply()?;

                if let Some(rollbacks) = &self.snapshot.rollbacks {
                    self.world.insert_resource(rollbacks.clone_value());
                }

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
