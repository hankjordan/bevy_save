use bevy::{
    prelude::*,
    reflect::TypeRegistry,
    scene::DynamicEntity,
};

use crate::{
    checkpoint::Checkpoints,
    error::Error,
    prelude::*,
    serde::SnapshotSerializer,
    CloneReflect,
};

/// A collection of serializable entities and resources.
///
/// Can be serialized via [`SnapshotSerializer`](crate::serde::SnapshotSerializer) and deserialized via [`SnapshotDeserializer`](crate::serde::SnapshotDeserializer).
pub struct Snapshot {
    /// Entities contained in the snapshot.
    pub entities: Vec<DynamicEntity>,

    /// Resources contained in the snapshot.
    pub resources: Vec<Box<dyn PartialReflect>>,

    pub(crate) checkpoints: Option<Checkpoints>,
}

impl Snapshot {
    /// Returns a complete [`Snapshot`] of the current [`World`] state.
    ///
    /// Contains all saveable entities, resources, and [`Checkpoints`].
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
    ///     .extract_all_with_checkpoints()
    ///     .build();
    pub fn from_world(world: &World) -> Self {
        Self::builder(world).extract_all_with_checkpoints().build()
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
        Self {
            entities: self.entities.clone_reflect(registry),
            resources: self.resources.clone_reflect(registry),
            checkpoints: self.checkpoints.clone_reflect(registry),
        }
    }
}
