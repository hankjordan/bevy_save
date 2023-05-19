use std::collections::HashSet;

use bevy::{
    ecs::{
        entity::EntityMap,
        reflect::ReflectMapEntities,
        system::CommandQueue,
    },
    prelude::*,
};

use crate::{
    entity::SaveableEntity,
    prelude::*,
};

pub(crate) struct RawSnapshot {
    pub(crate) resources: Vec<Box<dyn Reflect>>,
    pub(crate) entities: Vec<SaveableEntity>,
}

impl RawSnapshot {
    pub(crate) fn default() -> Self {
        Self {
            resources: Vec::default(),
            entities: Vec::default(),
        }
    }
}

impl<'a> Applier<'a, &'a RawSnapshot> {
    pub(crate) fn apply(self) -> Result<(), SaveableError> {
        let registry_arc = self.world.resource::<AppTypeRegistry>().clone();
        let registry = registry_arc.read();

        // Resources

        for resource in &self.snapshot.resources {
            let reg = registry
                .get_with_name(resource.type_name())
                .ok_or_else(|| SaveableError::UnregisteredType {
                    type_name: resource.type_name().to_string(),
                })?;

            let data = reg.data::<ReflectResource>().ok_or_else(|| {
                SaveableError::UnregisteredResource {
                    type_name: resource.type_name().to_string(),
                }
            })?;

            data.insert(self.world, resource.as_reflect());

            if let Some(mapper) = reg.data::<ReflectMapEntities>() {
                mapper
                    .map_entities(self.world, &self.map)
                    .map_err(SaveableError::MapEntitiesError)?;
            }
        }

        // Entities

        let despawn_default = self
            .world
            .get_resource::<AppDespawnMode>()
            .cloned()
            .unwrap_or_default();

        let despawn = self.despawn.as_ref().unwrap_or(&despawn_default);

        match despawn {
            DespawnMode::Missing | DespawnMode::MissingWith(_) => {
                let valid = self
                    .snapshot
                    .entities
                    .iter()
                    .map(|e| e.try_map(&self.map))
                    .collect::<HashSet<_>>();

                let mut invalid = self
                    .world
                    .iter_entities()
                    .map(|e| e.id())
                    .filter(|e| !valid.contains(e))
                    .collect::<Vec<_>>();

                if let DespawnMode::MissingWith(filter) = despawn {
                    let matches = filter.collect(self.world);
                    invalid.retain(|e| matches.contains(e));
                }

                for entity in invalid {
                    self.world.despawn(entity);
                }
            }

            DespawnMode::Unmapped | DespawnMode::UnmappedWith(_) => {
                let valid = self
                    .snapshot
                    .entities
                    .iter()
                    .filter_map(|e| e.map(&self.map))
                    .collect::<HashSet<_>>();

                let mut invalid = self
                    .world
                    .iter_entities()
                    .map(|e| e.id())
                    .filter(|e| !valid.contains(e))
                    .collect::<Vec<_>>();

                if let DespawnMode::UnmappedWith(filter) = despawn {
                    let matches = filter.collect(self.world);
                    invalid.retain(|e| matches.contains(e));
                }

                for entity in invalid {
                    self.world.despawn(entity);
                }
            }
            DespawnMode::All => {
                let invalid = self
                    .world
                    .iter_entities()
                    .map(|e| e.id())
                    .collect::<Vec<_>>();

                for entity in invalid {
                    self.world.despawn(entity);
                }
            }
            DespawnMode::AllWith(filter) => {
                let invalid = filter.collect(self.world);

                for entity in invalid {
                    self.world.despawn(entity);
                }
            }
            DespawnMode::None => {}
        }

        let mapping_default = self
            .world
            .get_resource::<AppMappingMode>()
            .cloned()
            .unwrap_or_default();

        let mapping = self.mapping.as_ref().unwrap_or(&mapping_default);

        let fallback = if let MappingMode::Simple = &mapping {
            let mut fallback = EntityMap::default();

            for entity in self.world.iter_entities() {
                fallback.insert(Entity::from_raw(entity.id().index()), entity.id());
            }

            fallback
        } else {
            EntityMap::default()
        };

        let mut spawned = Vec::new();

        // Apply snapshot entities
        for saved in &self.snapshot.entities {
            let index = saved.entity;

            let entity = saved
                .map(&self.map)
                .or_else(|| fallback.get(Entity::from_raw(index)).ok())
                .unwrap_or_else(|| self.world.spawn_empty().id());

            spawned.push(entity);

            let entity_mut = &mut self.world.entity_mut(entity);

            for component in &saved.components {
                let reg = registry
                    .get_with_name(component.type_name())
                    .ok_or_else(|| SaveableError::UnregisteredType {
                        type_name: component.type_name().to_string(),
                    })?;

                let data = reg.data::<ReflectComponent>().ok_or_else(|| {
                    SaveableError::UnregisteredComponent {
                        type_name: component.type_name().to_string(),
                    }
                })?;

                data.apply_or_insert(entity_mut, &**component);
            }
        }

        // ReflectMapEntities
        for reg in registry.iter() {
            if let Some(mapper) = reg.data::<ReflectMapEntities>() {
                mapper
                    .map_entities(self.world, &self.map)
                    .map_err(SaveableError::MapEntitiesError)?;
            }
        }

        // Entity hook
        if let Some(hook) = &self.hook {
            let mut queue = CommandQueue::default();
            let mut commands = Commands::new(&mut queue, self.world);

            for entity in spawned {
                let entity_ref = self.world.entity(entity);
                let mut entity_mut = commands.entity(entity);

                hook(&entity_ref, &mut entity_mut);
            }

            queue.apply(self.world);
        }

        Ok(())
    }
}

impl CloneReflect for RawSnapshot {
    fn clone_value(&self) -> Self {
        Self {
            resources: self.resources.clone_value(),
            entities: self.entities.iter().map(|e| e.clone_value()).collect(),
        }
    }
}
