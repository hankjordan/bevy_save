use std::collections::BTreeMap;

use bevy::{
    ecs::{
        entity::EntityMap,
        reflect::ReflectMapEntities,
    },
    prelude::*,
    reflect::TypeRegistration,
};

use crate::prelude::*;

/// A collection of serializable dynamic entities, each with its own run-time defined set of components.
///
/// Similar to `DynamicScene` but is filterable and only returns components registered with the [`SaveableRegistry`].
pub struct SaveableScene {
    /// The entities and their saveable components
    pub entities: Vec<SaveableEntity>,
}

impl SaveableScene {
    /// Creates a [`SaveableScene`] containing all saveable entities and components.
    pub fn from_world(world: &World) -> Self {
        Self::from_world_with_filter(world, |_| true)
    }

    /// Creates a [`SaveableScene`] containing all saveable entities and components matching the filter.
    pub fn from_world_with_filter<F>(world: &World, filter: F) -> Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        let type_registry = world.resource::<AppTypeRegistry>();
        let registry = type_registry.read();

        let saveables = world.resource::<SaveableRegistry>();

        let mut extracted = BTreeMap::new();

        for entity in world.iter_entities().map(|entity| entity.id()) {
            let index = entity.index();

            if extracted.contains_key(&index) {
                continue;
            }

            let mut entry = SaveableEntity {
                entity: index,
                components: Vec::new(),
            };

            let entity = world.entity(entity);

            for component_id in entity.archetype().components() {
                let reflect = world
                    .components()
                    .get_info(component_id)
                    .filter(|info| saveables.contains(info.name()))
                    .and_then(|info| info.type_id())
                    .and_then(|id| registry.get(id))
                    .filter(&filter)
                    .and_then(|reg| reg.data::<ReflectComponent>())
                    .and_then(|reflect| reflect.reflect(entity));

                if let Some(reflect) = reflect {
                    entry.components.push(reflect.clone_value());
                }
            }

            extracted.insert(index, entry);
        }

        Self {
            entities: extracted.into_values().collect(),
        }
    }

    /// Apply the [`SaveableScene`] to the [`World`].
    ///
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply(&self, world: &mut World) -> Result<(), SaveableError> {
        self.apply_with_map(world, &mut world.entity_map())
    }

    /// Apply the [`SaveableScene`] to the [`World`], mapping entities to new ids with the [`EntityMap`].
    ///
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply_with_map(
        &self,
        world: &mut World,
        map: &mut EntityMap,
    ) -> Result<(), SaveableError> {
        let registry_arc = world.resource::<AppTypeRegistry>().clone();
        let registry = registry_arc.read();

        for scene_entity in &self.entities {
            let entity = *map
                .entry(Entity::from_raw(scene_entity.entity))
                .or_insert_with(|| world.spawn_empty().id());

            let entity_mut = &mut world.entity_mut(entity);

            for component in &scene_entity.components {
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

        for reg in registry.iter() {
            if let Some(mapper) = reg.data::<ReflectMapEntities>() {
                mapper
                    .map_entities(world, map)
                    .map_err(SaveableError::MapEntitiesError)?;
            }
        }

        Ok(())
    }
}

impl CloneReflect for SaveableScene {
    fn clone_value(&self) -> Self {
        let mut entities = Vec::new();

        for entity in &self.entities {
            entities.push(entity.clone_value());
        }

        Self { entities }
    }
}
