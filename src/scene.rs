use std::collections::BTreeMap;

use bevy::{
    ecs::component::ComponentInfo,
    prelude::*,
    scene::DynamicEntity,
};

pub struct DynamicScene2 {
    pub entities: Vec<DynamicEntity>,
}

impl DynamicScene2 {
    pub fn from_world(world: &World) -> Self {
        Self::from_world_with_filter(world, |_| true)
    }

    pub fn from_world_with_filter<F>(world: &World, filter: F) -> Self
    where
        F: Fn(&&ComponentInfo) -> bool,
    {
        let type_registry = world.resource::<AppTypeRegistry>();
        let registry = type_registry.read();

        let mut extracted = BTreeMap::new();

        for entity in world.iter_entities().map(|entity| entity.id()) {
            let index = entity.index();

            if extracted.contains_key(&index) {
                continue;
            }

            let mut entry = DynamicEntity {
                entity: index,
                components: Vec::new(),
            };

            let entity = world.entity(entity);

            for component_id in entity.archetype().components() {
                let reflect = world
                    .components()
                    .get_info(component_id)
                    .filter(&filter)
                    .and_then(|info| registry.get(info.type_id().unwrap()))
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
}

impl From<DynamicScene2> for bevy::scene::DynamicScene {
    fn from(scene: DynamicScene2) -> Self {
        Self {
            entities: scene.entities,
        }
    }
}
