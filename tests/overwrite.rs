use bevy::prelude::*;
use bevy_save::prelude::*;

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Eq)]
#[reflect(Component)]
struct Collect {
    data: Vec<u32>,
}

#[test]
fn test_collect() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(SavePlugins);

    app.register_type::<Collect>();
    app.register_type::<Vec<u32>>();

    let world = app.world_mut();

    let entity = world
        .spawn_empty()
        .insert(Collect { data: Vec::new() })
        .id();

    world
        .entity_mut(entity)
        .get_mut::<Collect>()
        .unwrap()
        .data
        .push(1);

    assert_eq!(
        world.entity(entity).get::<Collect>(),
        Some(&Collect { data: vec![1] })
    );
    assert_eq!(world.iter_entities().count(), 1);

    let snapshot = Snapshot::builder(world).extract_entity(entity).build();

    world
        .entity_mut(entity)
        .get_mut::<Collect>()
        .unwrap()
        .data
        .push(2);

    assert_eq!(
        world.entity(entity).get::<Collect>(),
        Some(&Collect { data: vec![1, 2] })
    );

    snapshot
        .applier(world)
        .entity_map(&mut [(entity, entity)].into_iter().collect())
        .apply()
        .unwrap();

    assert_eq!(
        world.entity(entity).get::<Collect>(),
        Some(&Collect { data: vec![1] })
    );
    assert_eq!(world.iter_entities().count(), 1);
}

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Eq)]
#[reflect(Component)]
struct Basic {
    data: u32,
}

#[test]
fn test_basic() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(SavePlugins);

    app.register_type::<Basic>();

    let world = app.world_mut();

    let entity = world.spawn_empty().insert(Basic { data: 0 }).id();

    assert_eq!(
        world.entity(entity).get::<Basic>(),
        Some(&Basic { data: 0 })
    );
    assert_eq!(world.iter_entities().count(), 1);

    world.entity_mut(entity).get_mut::<Basic>().unwrap().data = 1;

    let snapshot = Snapshot::builder(world).extract_entity(entity).build();

    world.entity_mut(entity).get_mut::<Basic>().unwrap().data = 2;

    assert_eq!(
        world.entity(entity).get::<Basic>(),
        Some(&Basic { data: 2 })
    );

    snapshot
        .applier(world)
        .entity_map(&mut [(entity, entity)].into_iter().collect())
        .apply()
        .unwrap();

    assert_eq!(
        world.entity(entity).get::<Basic>(),
        Some(&Basic { data: 1 })
    );
    assert_eq!(world.iter_entities().count(), 1);
}
