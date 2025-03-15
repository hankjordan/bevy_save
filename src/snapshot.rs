use bevy::{
    prelude::*,
    reflect::TypeRegistry,
    scene::DynamicEntity,
};

use crate::{
    CloneReflect,
    Error,
    Rollbacks,
    SnapshotApplier,
    SnapshotBuilder,
    SnapshotSerializer,
};

/// A collection of serializable entities and resources.
///
/// Can be serialized via [`SnapshotSerializer`](crate::SnapshotSerializer) and deserialized via [`SnapshotDeserializer`](crate::SnapshotDeserializer).
pub struct Snapshot {
    /// Entities contained in the snapshot.
    pub entities: Vec<DynamicEntity>,

    /// Resources contained in the snapshot.
    pub resources: Vec<Box<dyn PartialReflect>>,

    pub(crate) rollbacks: Option<Rollbacks>,
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
    /// # let world = app.world_mut();
    /// Snapshot::builder(world)
    ///     .extract_all_with_rollbacks()
    ///     .build();
    pub fn from_world(world: &World) -> Self {
        Self::builder(world).extract_all_with_rollbacks().build()
    }

    /// Create a [`SnapshotBuilder`] from the [`World`], allowing you to create partial or filtered snapshots.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_save::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = app.world_mut();
    /// Snapshot::builder(world)
    ///     // Extract all matching entities and resources
    ///     .extract_all()
    ///
    ///     // Clear all extracted entities without any components
    ///     .clear_empty()
    ///
    ///     // Build the `Snapshot`
    ///     .build();
    /// ```
    pub fn builder(world: &World) -> SnapshotBuilder {
        SnapshotBuilder::snapshot(world)
    }

    /// Apply the [`Snapshot`] to the [`World`], using default applier settings.
    ///
    /// # Errors
    /// If a type included in the [`Snapshot`] has not been registered with the type registry.
    pub fn apply(&self, world: &mut World) -> Result<(), Error> {
        self.applier(world).apply()
    }

    /// Create a [`SnapshotApplier`] from the [`Snapshot`] and the [`World`].
    ///
    /// This allows you to specify an entity map, hook, etc.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_save::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = app.world_mut();
    /// # let parent = Entity::from_raw(0);
    /// let snapshot = Snapshot::from_world(world);
    ///
    /// snapshot
    ///     .applier(world)
    ///     .hook(move |entity, cmds| {
    ///         // You can use the hook to add, get, or remove Components
    ///         if !entity.contains::<Parent>() {
    ///             cmds.set_parent(parent);
    ///         }
    ///     })
    ///     .apply();
    /// ```
    pub fn applier<'a>(&'a self, world: &'a mut World) -> SnapshotApplier<'a> {
        SnapshotApplier::new(self, world)
    }

    /// Create a [`SnapshotSerializer`] from the [`Snapshot`] and the [`TypeRegistry`].
    pub fn serializer<'a>(&'a self, registry: &'a TypeRegistry) -> SnapshotSerializer<'a> {
        SnapshotSerializer {
            snapshot: self,
            registry,
        }
    }
}

impl CloneReflect for Snapshot {
    fn clone_reflect(&self, registry: &TypeRegistry) -> Self {
        #[allow(clippy::borrowed_box)]
        let clone_from_reflect = |value: &Box<dyn PartialReflect>| {
            registry
                .get(value.get_represented_type_info().unwrap().type_id())
                .and_then(|r| {
                    r.data::<ReflectFromReflect>()
                        .and_then(|fr| fr.from_reflect(value.as_partial_reflect()))
                        .map(|fr| fr.into_partial_reflect())
                })
                .unwrap_or_else(|| value.clone_value())
        };

        Self {
            entities: self
                .entities
                .iter()
                .map(|d| DynamicEntity {
                    entity: d.entity,
                    components: d.components.iter().map(&clone_from_reflect).collect(),
                })
                .collect(),
            resources: self.resources.iter().map(&clone_from_reflect).collect(),
            rollbacks: self.rollbacks.as_ref().map(|r| r.clone_reflect(registry)),
        }
    }
}
