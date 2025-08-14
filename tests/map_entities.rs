use std::collections::HashMap;

use bevy::{
    ecs::{
        entity::{
            EntityHashMap,
            MapEntities,
        },
        reflect::ReflectMapEntities,
    },
    prelude::*,
};
use bevy_save::prelude::*;

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
struct SimpleComponent {
    #[entities]
    target: Entity,
}

#[derive(Resource, Reflect, MapEntities, Clone, Debug, PartialEq)]
#[reflect(Resource, MapEntities)]
struct SimpleResource {
    #[entities]
    target: Entity,
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, MapEntities)]
struct ExampleComponent {
    targets: HashMap<u32, Entity>,
}

impl MapEntities for ExampleComponent {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for entity in self.targets.values_mut() {
            *entity = entity_mapper.get_mapped(*entity);
        }
    }
}

#[derive(Resource, Reflect, Clone, Debug, PartialEq)]
#[reflect(Resource, MapEntities)]
struct ExampleResource {
    targets: HashMap<u32, Entity>,
}

impl MapEntities for ExampleResource {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for entity in self.targets.values_mut() {
            *entity = entity_mapper.get_mapped(*entity);
        }
    }
}

fn init_app() -> App {
    let mut app = App::new();

    app.register_type::<SimpleComponent>()
        .register_type::<SimpleResource>()
        .register_type::<ExampleComponent>()
        .register_type::<ExampleResource>();

    app
}

#[test]
fn test_map_entities_components() {
    let mut app = init_app();

    let orig = ExampleComponent {
        targets: [
            (0, Entity::from_raw(10)),
            (1, Entity::from_raw(20)),
            (2, Entity::from_raw(30)),
        ]
        .into_iter()
        .collect(),
    };

    app.world_mut().spawn(orig.clone());

    let snap_a = Snapshot::from_world(app.world());

    let mut map: EntityHashMap<Entity> = [
        (Entity::from_raw(10), Entity::from_raw(100)),
        (Entity::from_raw(20), Entity::from_raw(200)),
        (Entity::from_raw(30), Entity::from_raw(300)),
    ]
    .into_iter()
    .collect();

    snap_a
        .applier(app.world_mut())
        .entity_map(&mut map)
        .despawn::<With<ExampleComponent>>()
        .apply()
        .expect("Failed to apply");

    let snap_b = Snapshot::from_world(app.world());

    assert_eq!(snap_a.entities().len(), 1);
    assert_eq!(snap_b.entities().len(), 1);

    assert!(snap_a.resources().is_empty());
    assert!(snap_b.resources().is_empty());

    let a = ExampleComponent::from_reflect(
        &**snap_a
            .entities()
            .first()
            .expect("Could not find entity")
            .components
            .first()
            .expect("Could not find component"),
    )
    .expect("FromReflect failed");

    let b = ExampleComponent::from_reflect(
        &**snap_b
            .entities()
            .first()
            .expect("Could not find entity")
            .components
            .first()
            .expect("Could not find component"),
    )
    .expect("FromReflect failed");

    assert_eq!(a, orig);

    assert_eq!(b, ExampleComponent {
        targets: [
            (0, Entity::from_raw(100)),
            (1, Entity::from_raw(200)),
            (2, Entity::from_raw(300))
        ]
        .into_iter()
        .collect()
    });
}

#[test]
fn test_map_entities_resources() {
    let mut app = init_app();

    let orig = ExampleResource {
        targets: [
            (0, Entity::from_raw(10)),
            (1, Entity::from_raw(20)),
            (2, Entity::from_raw(30)),
        ]
        .into_iter()
        .collect(),
    };

    app.insert_resource(orig.clone());

    let snap_a = Snapshot::from_world(app.world());

    let mut map: EntityHashMap<Entity> = [
        (Entity::from_raw(10), Entity::from_raw(100)),
        (Entity::from_raw(20), Entity::from_raw(200)),
        (Entity::from_raw(30), Entity::from_raw(300)),
    ]
    .into_iter()
    .collect();

    snap_a
        .applier(app.world_mut())
        .entity_map(&mut map)
        .apply()
        .expect("Failed to apply");

    let snap_b = Snapshot::from_world(app.world());

    assert!(snap_a.entities().is_empty());
    assert!(snap_b.entities().is_empty());

    assert_eq!(snap_a.resources().len(), 1);
    assert_eq!(snap_b.resources().len(), 1);

    let a = ExampleResource::from_reflect(
        &**snap_a.resources().first().expect("Could not find resource"),
    )
    .expect("FromReflect failed");

    let b = ExampleResource::from_reflect(
        &**snap_b.resources().first().expect("Could not find resource"),
    )
    .expect("FromReflect failed");

    assert_eq!(a, orig);

    assert_eq!(b, ExampleResource {
        targets: [
            (0, Entity::from_raw(100)),
            (1, Entity::from_raw(200)),
            (2, Entity::from_raw(300))
        ]
        .into_iter()
        .collect()
    });
}

#[test]
fn test_map_entities_simple() {
    let mut app = init_app();

    let orig_comp = SimpleComponent {
        target: Entity::from_raw(10),
    };
    let orig_res = SimpleResource {
        target: Entity::from_raw(10),
    };

    app.world_mut().spawn(orig_comp.clone());
    app.insert_resource(orig_res.clone());

    let snap_a = Snapshot::from_world(app.world());

    let mut map: EntityHashMap<Entity> = [
        (Entity::from_raw(10), Entity::from_raw(100)),
        (Entity::from_raw(20), Entity::from_raw(200)),
        (Entity::from_raw(30), Entity::from_raw(300)),
    ]
    .into_iter()
    .collect();

    snap_a
        .applier(app.world_mut())
        .entity_map(&mut map)
        .despawn::<With<SimpleComponent>>()
        .apply()
        .expect("Failed to apply");

    let snap_b = Snapshot::from_world(app.world());

    assert_eq!(snap_a.entities().len(), 1);
    assert_eq!(snap_b.entities().len(), 1);

    assert_eq!(snap_a.resources().len(), 1);
    assert_eq!(snap_b.resources().len(), 1);

    let comp_a = SimpleComponent::from_reflect(
        &**snap_a
            .entities()
            .first()
            .expect("Could not find entity")
            .components
            .first()
            .expect("Could not find component"),
    )
    .expect("FromReflect failed");

    let comp_b = SimpleComponent::from_reflect(
        &**snap_b
            .entities()
            .first()
            .expect("Could not find entity")
            .components
            .first()
            .expect("Could not find component"),
    )
    .expect("FromReflect failed");

    assert_eq!(comp_a, orig_comp);
    assert_eq!(comp_b, SimpleComponent {
        target: Entity::from_raw(100)
    });

    let res_a = SimpleResource::from_reflect(
        &**snap_a.resources().first().expect("Could not find resource"),
    )
    .expect("FromReflect failed");

    let res_b = SimpleResource::from_reflect(
        &**snap_b.resources().first().expect("Could not find resource"),
    )
    .expect("FromReflect failed");

    assert_eq!(res_a, orig_res);
    assert_eq!(res_b, SimpleResource {
        target: Entity::from_raw(100)
    });
}
