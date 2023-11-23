use bevy::{
    prelude::*,
    scene::DynamicEntity,
};

use crate::{
    CloneReflect,
    Error,
    Rollbacks,
    SnapshotApplier,
    SnapshotBuilder,
};

/// A collection of serializable entities and resources.
///
/// Can be serialized via [`SnapshotSerializer`] and deserialized via [`SnapshotDeserializer`].
pub struct Snapshot {
    /// Entities contained in the snapshot.
    pub entities: Vec<DynamicEntity>,

    /// Resources contained in the snapshot.
    pub resources: Vec<Box<dyn Reflect>>,

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
    /// # let world = &mut app.world;
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
    /// # let world = &mut app.world;
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
    /// # let world = &mut app.world;
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
    pub fn applier<'a>(&'a self, world: &'a mut World) -> SnapshotApplier<'_> {
        SnapshotApplier::new(self, world)
    }
}

impl CloneReflect for Snapshot {
    fn clone_value(&self) -> Self {
        Self {
            entities: self.entities.iter().map(|e| e.clone_value()).collect(),
            resources: self.resources.clone_value(),
            rollbacks: self.rollbacks.clone_value(),
        }
    }
}
