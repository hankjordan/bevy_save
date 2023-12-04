use std::marker::PhantomData;

use bevy::{
    ecs::{
        query::ReadOnlyWorldQuery,
        system::{
            CommandQueue,
            EntityCommands,
        },
    },
    prelude::*,
    utils::HashMap,
};

use crate::{
    prelude::*,
    typed::{
        extract::{
            ExtractComponent,
            ExtractMapEntities,
            ExtractResource,
            Extractable,
        },
        snapshot::Extracted,
    },
    Hook,
};

/// [`SnapshotApplier`] lets you configure how a snapshot will be applied to the [`World`].
pub struct SnapshotApplier<
    'w,
    C: Extractable,
    R: Extractable,
    D = (),
    H = fn(&EntityRef, &mut EntityCommands),
> {
    snapshot: &'w Snapshot<C, R>,
    world: &'w mut World,
    entity_map: Option<&'w mut HashMap<Entity, Entity>>,
    despawn: Option<PhantomData<D>>,
    hook: Option<H>,
}

impl<'w, C, R> SnapshotApplier<'w, C, R>
where
    C: Extractable,
    R: Extractable,
{
    /// Create a new [`SnapshotApplier`] with from the world and snapshot.
    pub fn new(snapshot: &'w Snapshot<C, R>, world: &'w mut World) -> Self {
        Self {
            snapshot,
            world,
            entity_map: None,
            despawn: None,
            hook: None,
        }
    }
}

impl<'w, C, R, D, H> SnapshotApplier<'w, C, R, D, H>
where
    C: Extractable,
    R: Extractable,
{
    /// Providing an entity map allows you to map ids of spawned entities and see what entities have been spawned.
    pub fn entity_map(mut self, entity_map: &'w mut HashMap<Entity, Entity>) -> Self {
        self.entity_map = Some(entity_map);
        self
    }

    /// Change how the snapshot affects existing entities while applying.
    pub fn despawn<F: ReadOnlyWorldQuery + 'static>(self) -> SnapshotApplier<'w, C, R, F, H> {
        SnapshotApplier {
            snapshot: self.snapshot,
            world: self.world,
            entity_map: self.entity_map,
            despawn: Some(PhantomData),
            hook: self.hook,
        }
    }

    /// Add a [`Hook`] that will run for each entity after applying.
    pub fn hook<F: Hook + 'static>(self, hook: F) -> SnapshotApplier<'w, C, R, D, F> {
        SnapshotApplier {
            snapshot: self.snapshot,
            world: self.world,
            entity_map: self.entity_map,
            despawn: self.despawn,
            hook: Some(hook),
        }
    }
}

impl<'w, C, R, D, H> SnapshotApplier<'w, C, R, D, H>
where
    C: ExtractComponent + ExtractMapEntities,
    R: ExtractResource + ExtractMapEntities,
    D: ReadOnlyWorldQuery,
    H: Hook,
{
    /// Apply the [`Snapshot`] to the [`World`].
    ///
    /// # Panics
    /// If `type_registry` is not set or the [`AppTypeRegistry`] resource does not exist.
    ///
    /// # Errors
    /// If a type included in the [`Snapshot`] has not been registered with the type registry.
    pub fn apply(self) {
        let mut default_entity_map = HashMap::new();
        let entity_map = self.entity_map.unwrap_or(&mut default_entity_map);

        // Despawn entities
        if self.despawn.is_some() {
            let invalid = self
                .world
                .query_filtered::<Entity, D>()
                .iter(self.world)
                .collect::<Vec<_>>();

            for entity in invalid {
                self.world.despawn(entity);
            }
        }

        // Resources
        R::apply(&self.snapshot.resources.0, self.world);

        // Entities
        for (saved, Extracted(components)) in &self.snapshot.entities.0 {
            let entity = *entity_map
                .entry(*saved)
                .or_insert_with(|| self.world.spawn_empty().id());

            C::apply(components, &mut self.world.entity_mut(entity));

            // TODO: Map entities
        }

        // TODO: Map entities

        // Entity hook
        if let Some(hook) = &self.hook {
            let mut queue = CommandQueue::default();
            let mut commands = Commands::new(&mut queue, self.world);

            for (_, entity) in entity_map {
                let entity_ref = self.world.entity(*entity);
                let mut entity_mut = commands.entity(*entity);

                hook(&entity_ref, &mut entity_mut);
            }

            queue.apply(self.world);
        }
    }
}
