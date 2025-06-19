use bevy::{
    ecs::{
        entity::{
            EntityHashMap,
            MapEntities,
        },
        reflect::ReflectMapEntities,
    },
    prelude::*,
    reflect::TypeRegistry,
};
use bevy_save::prelude::*;

#[derive(Component, Reflect, Clone, MapEntities)]
#[relationship(relationship_target = Children)]
#[reflect(Component)]
pub struct ChildOf {
    #[relationship]
    #[entities]
    pub parent: Entity,
}

#[derive(Component, Reflect, Clone, MapEntities)]
#[relationship_target(relationship = ChildOf)]
#[reflect(Component)]
pub struct Children(#[entities] Vec<Entity>);

fn empty_app() -> App {
    let mut app = App::new();

    app //
        .add_plugins((MinimalPlugins, SavePlugins))
        .register_type::<IsChild>()
        .register_type::<ChildPrefab>()
        .register_type::<ChildOf>()
        .register_type::<Children>();

    app
}

fn init_app() -> (App, Vec<Entity>) {
    let mut app = empty_app();

    let world = app.world_mut();

    let parent = world.spawn_empty().id();

    let a = world.spawn((IsChild, ChildOf { parent })).id();
    let b = world.spawn((IsChild, ChildOf { parent })).id();
    let c = world.spawn((IsChild, ChildOf { parent })).id();

    (app, vec![parent, a, b, c])
}

fn dump_snapshot(registry: &TypeRegistry, snapshot: &Snapshot) {
    println!(
        "{}",
        serde_json::to_string_pretty(&snapshot.serializer(registry))
            .expect("Failed to serialize snapshot")
    );
}

fn dump_entities(world: &World) {
    for entity in world.iter_entities() {
        println!(
            "Entity {:?}: {:?}",
            entity.id(),
            world
                .inspect_entity(entity.id())
                .expect("Invalid entity")
                .map(|i| i.name())
                .collect::<Vec<_>>()
        );
    }
}

fn check_children<F>(snapshot: &Snapshot, entities: &[Entity], applier: F)
where
    F: for<'a> Fn(&'a mut World, &'a Snapshot) -> SnapshotApplier<'a>,
{
    // Check behavior without mapping
    let mut app = empty_app();
    let world = app.world_mut();

    applier(world, snapshot)
        .apply()
        .expect("Failed to apply snapshot");

    dump_entities(world);

    let (parent, expected) = world
        .query::<(Entity, &Children)>()
        .single(world)
        .expect("Could not find root");

    println!("old root {:?} -> new root {:?}", entities[0], parent);

    let expected = expected.0.clone();

    let children = world
        .query::<(Entity, &ChildOf)>()
        .iter(world)
        .map(|(e, c)| (e, c.parent))
        .collect::<Vec<_>>();

    assert_eq!(children.len(), 3);

    for (child, child_parent) in children {
        println!("Child: {:?} (of {:?})", child, child_parent);
        assert!(expected.contains(&child));
        assert_eq!(parent, child_parent);
    }

    // Check behavior after mapping
    let mut app = empty_app();
    let world = app.world_mut();

    for _ in 0..20 {
        world.spawn_empty();
    }

    let mut map: EntityHashMap<Entity> = vec![
        (entities[1], world.spawn_empty().id()),
        (entities[0], world.spawn_empty().id()),
    ]
    .into_iter()
    .collect();

    applier(world, snapshot)
        .entity_map(&mut map)
        .apply()
        .expect("Failed to apply snapshot");

    dump_entities(world);

    let (parent, expected) = world
        .query::<(Entity, &Children)>()
        .single(world)
        .expect("Could not find root");

    println!("old root {:?} -> new root {:?}", entities[0], parent);

    let expected = expected.0.clone();

    let children = world
        .query::<(Entity, &ChildOf)>()
        .iter(world)
        .map(|(e, c)| (e, c.parent))
        .collect::<Vec<_>>();

    assert_eq!(children.len(), 3);

    for (child, child_parent) in children {
        println!("Child: {:?} (of {:?})", child, child_parent);
        assert!(expected.contains(&child));
        assert_eq!(parent, child_parent);
    }
}

#[test]
fn test_relationships() {
    // Check expected, default behavior
    let (app, entities) = init_app();
    let registry = app.world().resource::<AppTypeRegistry>().read();

    let expected = app
        .world()
        .entity(entities[0])
        .get::<Children>()
        .expect("Could not find Children component on root")
        .clone()
        .0;

    assert_eq!(expected, vec![entities[1], entities[2], entities[3]]);

    let snapshot = Snapshot::builder(app.world())
        .extract_all_entities()
        .build();

    dump_snapshot(&registry, &snapshot);

    check_children(&snapshot, &entities, |w, s| s.applier(w));
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct IsChild;

#[derive(Reflect, MapEntities)]
#[reflect(MapEntities)]
pub struct ChildPrefab {
    #[entities]
    parent: Entity,
}

impl Prefab for ChildPrefab {
    type Marker = IsChild;

    fn spawn(self, target: Entity, world: &mut World) {
        world.entity_mut(target).insert(ChildOf {
            parent: self.parent,
        });
    }

    fn extract(builder: SnapshotBuilder) -> SnapshotBuilder {
        builder.extract_prefab(|entity| {
            let parent = entity.get::<ChildOf>()?.parent;

            Some(Self { parent })
        })
    }
}

#[test]
fn test_prefab_relationships() {
    let (app, entities) = init_app();
    let registry = app.world().resource::<AppTypeRegistry>().read();

    let snapshot = Snapshot::builder(app.world())
        .extract_entity(entities[0])
        .extract_all_prefabs::<ChildPrefab>()
        .build();

    dump_snapshot(&registry, &snapshot);

    check_children(&snapshot, &entities, |w, s| {
        s.applier(w).prefab::<ChildPrefab>()
    });
}
