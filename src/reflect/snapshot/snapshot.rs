use bevy::{
    prelude::*,
    reflect::TypeRegistry,
    scene::DynamicEntity,
};

use crate::{
    CloneReflect,
    error::Error,
    prelude::*,
    reflect::SnapshotSerializer,
};

/// A collection of serializable entities and resources.
///
/// Can be serialized via [`SnapshotSerializer`](crate::reflect::SnapshotSerializer) and deserialized via [`SnapshotDeserializer`](crate::reflect::SnapshotDeserializer).
pub struct Snapshot {
    /// Entities contained in the snapshot.
    pub entities: Vec<DynamicEntity>,

    /// Resources contained in the snapshot.
    pub resources: Vec<Box<dyn PartialReflect>>,

    #[cfg(feature = "checkpoints")]
    pub(crate) checkpoints: Option<crate::reflect::checkpoint::Checkpoints>,
}

impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct DebugEntity<'a>(&'a DynamicEntity);

        impl std::fmt::Debug for DebugEntity<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("DynamicEntity")
                    .field("entity", &self.0.entity)
                    .field("components", &self.0.components)
                    .finish()
            }
        }

        let mut f = f.debug_struct("Snapshot");

        f.field(
            "entities",
            &self.entities.iter().map(DebugEntity).collect::<Vec<_>>(),
        )
        .field("resources", &self.resources);

        #[cfg(feature = "checkpoints")]
        f.field("checkpoints", &self.checkpoints);

        f.finish()
    }
}

impl Snapshot {
    /// Returns a complete [`Snapshot`] of the current [`World`] state.
    ///
    /// Contains all saveable entities, resources, and [`Checkpoints`](crate::reflect::checkpoint::Checkpoints).
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
        let b = Self::builder(world).extract_all();

        #[cfg(feature = "checkpoints")]
        let b = b.extract_checkpoints();

        b.build()
    }

    /// Create a [`BuilderRef`] from the [`World`], allowing you to create partial or filtered snapshots.
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
    pub fn builder(world: &World) -> BuilderRef {
        BuilderRef::new(world)
    }

    /// Apply the [`Snapshot`] to the [`World`], using default applier settings.
    ///
    /// # Errors
    /// If a type included in the [`Snapshot`] has not been registered with the type registry.
    pub fn apply(&self, world: &mut World) -> Result<(), Error> {
        self.applier(world).apply()
    }

    /// Create an [`ApplierRef`] from the [`Snapshot`] and the [`World`].
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
    ///         if !entity.contains::<ChildOf>() {
    ///             cmds.insert(ChildOf(parent));
    ///         }
    ///     })
    ///     .apply();
    /// ```
    pub fn applier<'w, 'i>(&'i self, world: &'w mut World) -> ApplierRef<'w, 'i> {
        ApplierRef::new(self, world)
    }

    /// Create a [`SnapshotSerializer`] from the [`Snapshot`] and the [`TypeRegistry`].
    pub fn serializer<'a>(&'a self, registry: &'a TypeRegistry) -> SnapshotSerializer<'a> {
        SnapshotSerializer::new(self, registry)
    }
}

impl CloneReflect for Snapshot {
    fn clone_reflect(&self, registry: &TypeRegistry) -> Self {
        Self {
            entities: self.entities.clone_reflect(registry),
            resources: self.resources.clone_reflect(registry),
            #[cfg(feature = "checkpoints")]
            checkpoints: self.checkpoints.clone_reflect(registry),
        }
    }
}
