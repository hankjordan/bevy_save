use bevy::{
    prelude::*,
    reflect::TypeRegistry,
    scene::DynamicEntity,
};

use crate::{
    error::Error,
    prelude::*,
    reflect::{
        EntityMap,
        ReflectMap,
        SnapshotDeserializer,
        SnapshotSerializer,
    },
};

/// A collection of serializable entities and resources.
///
/// Can be serialized via
/// [`SnapshotSerializer`](crate::reflect::SnapshotSerializer) and deserialized
/// via [`SnapshotDeserializer`](crate::reflect::SnapshotDeserializer).
#[derive(Reflect, Clone, Debug)]
#[reflect(Clone)]
#[type_path = "bevy_save"]
pub struct Snapshot {
    /// Entities contained in the snapshot.
    pub entities: EntityMap,

    /// Resources contained in the snapshot.
    pub resources: ReflectMap,
}

impl Snapshot {
    /// Returns a complete [`Snapshot`] of the current [`World`] state.
    ///
    /// Contains all saveable entities and resources.
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
    ///     .extract_all()
    ///     .build();
    pub fn from_world(world: &World) -> Self {
        Self::builder(world).extract_all().build()
    }

    /// Create a [`BuilderRef`] from the [`World`], allowing you to create
    /// partial or filtered snapshots.
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
}

impl Snapshot {
    /// Returns a reference to the slice of entities contained in the [`Snapshot`].
    pub fn entities(&self) -> &[DynamicEntity] {
        // SAFETY: DynamicEntity and bevy::scene::DynamicEntity are equivalent
        unsafe { &*(std::ptr::from_ref(self.entities.as_slice()) as *const _) }
    }

    /// Returns a reference to the slice of resources contained in the [`Snapshot`].
    pub fn resources(&self) -> &[Box<dyn PartialReflect>] {
        // SAFETY: DynamicValue and Box<dyn PartialReflect> are equivalent
        unsafe { &*(std::ptr::from_ref(self.resources.as_slice()) as *const _) }
    }
}

impl Snapshot {
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

    /// Create a [`SnapshotDeserializer`] from the [`TypeRegistry`].
    pub fn deserializer(registry: &TypeRegistry) -> SnapshotDeserializer<'_> {
        SnapshotDeserializer::new(registry)
    }
}
