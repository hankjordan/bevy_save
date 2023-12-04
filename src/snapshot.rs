use bevy::{
    prelude::*,
    scene::DynamicEntity,
};

use crate::{
    extract::{
        ExtractDeserialize,
        ExtractSerialize,
        Extractable,
    },
    CloneReflect,
    DynamicSnapshotApplier,
    DynamicSnapshotBuilder,
    Error,
    Rollbacks,
};

/// A dynamic collection of serializable entities and resources.
///
/// Can be serialized via [`SnapshotSerializer`](crate::SnapshotSerializer) and deserialized via [`SnapshotDeserializer`](crate::SnapshotDeserializer).
pub struct DynamicSnapshot {
    /// Entities contained in the snapshot.
    pub entities: Vec<DynamicEntity>,

    /// Resources contained in the snapshot.
    pub resources: Vec<Box<dyn Reflect>>,

    pub(crate) rollbacks: Option<Rollbacks>,
}

impl DynamicSnapshot {
    /// Returns a complete [`DynamicSnapshot`] of the current [`World`] state.
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
    /// DynamicSnapshot::builder(world)
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
    /// DynamicSnapshot::builder(world)
    ///     // Extract all matching entities and resources
    ///     .extract_all()
    ///
    ///     // Clear all extracted entities without any components
    ///     .clear_empty()
    ///
    ///     // Build the `Snapshot`
    ///     .build();
    /// ```
    pub fn builder(world: &World) -> DynamicSnapshotBuilder {
        DynamicSnapshotBuilder::snapshot(world)
    }

    /// Apply the [`Snapshot`] to the [`World`], using default applier settings.
    ///
    /// # Errors
    /// If a type included in the [`DynamicSnapshot`] has not been registered with the type registry.
    pub fn apply(&self, world: &mut World) -> Result<(), Error> {
        self.applier(world).apply()
    }

    /// Create a [`DynamicSnapshotApplier`] from the [`DynamicSnapshot`] and the [`World`].
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
    pub fn applier<'a>(&'a self, world: &'a mut World) -> DynamicSnapshotApplier<'_> {
        DynamicSnapshotApplier::new(self, world)
    }
}

impl CloneReflect for DynamicSnapshot {
    fn clone_value(&self) -> Self {
        Self {
            entities: self.entities.iter().map(|e| e.clone_value()).collect(),
            resources: self.resources.clone_value(),
            rollbacks: self.rollbacks.clone_value(),
        }
    }
}

// --------------------------------------------------------------------------------------------------------------------

/// A collection of entities and resources.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(
    serialize = "C: ExtractSerialize, R: ExtractSerialize",
    deserialize = "C: ExtractDeserialize, R: ExtractDeserialize"
))]
pub struct Snapshot<C: Extractable, R: Extractable> {
    /// Entities contained in the snapshot.
    pub entities: Entities<C>,

    /// Resources contained in the snapshot.
    pub resources: Extracted<R>,
}

/// Wrapper type allowing serialization and deserialization of extracted entities
pub struct Entities<C: Extractable>(pub Vec<(Entity, Extracted<C>)>);

/// Wrapper type allowing serialization and deserialization of extracted types
pub struct Extracted<E: Extractable>(pub E::Value);
