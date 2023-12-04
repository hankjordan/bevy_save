use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    prelude::*,
    typed::{
        extract::{
            ExtractComponent,
            ExtractResource,
        },
        snapshot::{
            Entities,
            Extracted,
        },
    },
};

/// A snapshot builder that can extract entities and resources from a [`World`].
pub struct SnapshotBuilder<'w, I, C, R> {
    pub(crate) world: &'w World,
    pub(crate) entities: I,
    pub(crate) _marker: PhantomData<(C, R)>,
}

impl<'w, I, C, R> SnapshotBuilder<'w, I, C, R>
where
    I: Iterator<Item = Entity> + 'w,
{
    /// Extract a single entity from the builder’s [`World`].
    pub fn extract_entity(
        self,
        entity: Entity,
    ) -> SnapshotBuilder<'w, impl Iterator<Item = Entity> + 'w, C, R> {
        SnapshotBuilder {
            world: self.world,
            entities: self.entities.chain([entity]),
            _marker: PhantomData,
        }
    }

    /// Extract the given entities from the builder’s [`World`].
    pub fn extract_entities<E>(
        self,
        entities: E,
    ) -> SnapshotBuilder<'w, impl Iterator<Item = Entity> + 'w, C, R>
    where
        E: Iterator<Item = Entity> + 'w,
    {
        SnapshotBuilder {
            world: self.world,
            entities: self.entities.chain(entities),
            _marker: PhantomData,
        }
    }

    /// Extract the entities matching the given filter from the builder’s [`World`].
    pub fn extract_entities_matching<F: Fn(&EntityRef) -> bool + 'w>(
        self,
        filter: F,
    ) -> SnapshotBuilder<'w, impl Iterator<Item = Entity> + 'w, C, R> {
        let entities = self.world.iter_entities().filter(filter).map(|e| e.id());
        self.extract_entities(entities)
    }

    /// Extract all entities from the builder’s [`World`].
    pub fn extract_all_entities(
        self,
    ) -> SnapshotBuilder<'w, impl Iterator<Item = Entity> + 'w, C, R> {
        let entities = self.world.iter_entities().map(|e| e.id());

        SnapshotBuilder {
            world: self.world,
            entities: self.entities.chain(entities),
            _marker: PhantomData,
        }
    }
}

impl<'w, I, C, R> SnapshotBuilder<'w, I, C, R> {
    /// Clear all extracted entities.
    pub fn clear_entities(self) -> SnapshotBuilder<'w, impl Iterator<Item = Entity> + 'w, C, R> {
        SnapshotBuilder {
            world: self.world,
            entities: [].into_iter(),
            _marker: PhantomData,
        }
    }
}

impl<'w, I, C, R> SnapshotBuilder<'w, I, C, R>
where
    I: Iterator<Item = Entity> + 'w,
    C: ExtractComponent,
    R: ExtractResource,
{
    /// This will extract all registered resources and all registered components on the given entities.
    pub fn build(self) -> Snapshot<C, R> {
        Snapshot {
            entities: Entities(
                self.entities
                    .map(|e| (e, Extracted(C::extract(&self.world.entity(e)))))
                    .collect(),
            ),
            resources: Extracted(R::extract(self.world)),
        }
    }
}
