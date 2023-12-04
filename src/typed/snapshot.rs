use bevy::prelude::*;

use crate::{
    prelude::*,
    typed::extract::{
        ExtractComponent,
        ExtractDeserialize,
        ExtractMapEntities,
        ExtractResource,
        ExtractSerialize,
        Extractable,
    },
};

/// A collection of entities and resources.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(
    serialize = "C: ExtractSerialize, R: ExtractSerialize",
    deserialize = "C: ExtractDeserialize, R: ExtractDeserialize"
))]
pub struct Snapshot<C: Extractable, R: Extractable> {
    pub(crate) entities: Entities<C>,
    pub(crate) resources: Extracted<R>,
}

pub(crate) struct Entities<C: Extractable>(pub Vec<(Entity, Extracted<C>)>);

pub(crate) struct Extracted<E: Extractable>(pub E::Value);

impl<C, R> Snapshot<C, R>
where
    C: Extractable,
    R: Extractable,
{
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
    pub fn applier<'w>(&'w self, world: &'w mut World) -> SnapshotApplier<'w, C, R> {
        SnapshotApplier::new(self, world)
    }
}

impl<C, R> Snapshot<C, R>
where
    C: ExtractComponent + ExtractMapEntities,
    R: ExtractResource + ExtractMapEntities,
{
    /// Apply the [`Snapshot`] to the [`World`], using default applier settings.
    pub fn apply(&self, world: &mut World) {
        self.applier(world).apply();
    }
}
