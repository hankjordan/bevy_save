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

    app.add_plugins(SavePlugins);

    app.register_saveable::<Collect>();
    app.register_type::<Vec<u32>>();

    let world = &mut app.world;

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

    let snapshot = world.snapshot();

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

    snapshot.applier(world).apply().unwrap();

    assert_eq!(
        world.entity(entity).get::<Collect>(),
        Some(&Collect { data: vec![1] })
    );
}

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Eq)]
#[reflect(Component)]
struct Basic {
    data: u32,
}

#[test]
fn test_basic() {
    let mut app = App::new();

    app.add_plugins(SavePlugins);

    app.register_saveable::<Basic>();

    let world = &mut app.world;

    let entity = world.spawn_empty().insert(Basic { data: 0 }).id();

    world.entity_mut(entity).get_mut::<Basic>().unwrap().data = 1;

    let snapshot = world.snapshot();

    world.entity_mut(entity).get_mut::<Basic>().unwrap().data = 2;

    snapshot.applier(world).apply().unwrap();

    assert_eq!(
        world.entity(entity).get::<Basic>(),
        Some(&Basic { data: 1 })
    );
}
