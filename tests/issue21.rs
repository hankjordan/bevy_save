use std::collections::HashMap;

use bevy::prelude::*;
use bevy_save::prelude::*;

#[derive(Component)]
struct Selectable;

#[test]
fn issue21() {
    let mut app = App::new();
    app.add_plugins(SavePlugins);

    let world = &mut app.world;

    world.spawn((Name::from("ABC"), Selectable));
    world.spawn((Name::from("DEF"), Selectable));
    world.spawn(Name::from("GHI"));
    world.spawn(Name::from("JKL"));
    world.spawn(());
    world.spawn(Selectable);

    assert!(world.despawn(Entity::from_bits(5))); // 5v0

    world.spawn(Selectable);

    assert!(world.despawn(Entity::from_bits((1 << 32) + 5))); // 5v1

    world.spawn(Selectable);

    let entities = world.iter_entities().map(|e| (e.id(), e.get::<Name>())).collect::<HashMap<_, _>>();

    assert_eq!(entities.get(&Entity::from_raw(0)), Some(&Some(&Name::from("ABC"))));
    assert_eq!(entities.get(&Entity::from_raw(1)), Some(&Some(&Name::from("DEF"))));
    assert_eq!(entities.get(&Entity::from_raw(2)), Some(&Some(&Name::from("GHI"))));
    assert_eq!(entities.get(&Entity::from_raw(3)), Some(&Some(&Name::from("JKL"))));
    assert_eq!(entities.get(&Entity::from_raw(4)), Some(&None));
    assert_eq!(entities.get(&Entity::from_bits((2 << 32) + 5)), Some(&None));

    drop(entities);

    let snapshot = Snapshot::builder(world)
        .extract_all_entities()
        .build();    

    let filter = <dyn Filter>::new::<With<Selectable>>();

    snapshot
        .applier(world)
        .despawn(bevy_save::DespawnMode::MissingWith(Box::new(filter)))
        .apply()
        .unwrap();

    let entities = world.iter_entities().map(|e| (e.id(), e.get::<Name>())).collect::<HashMap<_, _>>();

    assert_eq!(entities.get(&Entity::from_raw(0)), Some(&Some(&Name::from("ABC"))));
    assert_eq!(entities.get(&Entity::from_raw(1)), Some(&Some(&Name::from("DEF"))));
    assert_eq!(entities.get(&Entity::from_raw(2)), Some(&Some(&Name::from("GHI"))));
    assert_eq!(entities.get(&Entity::from_raw(3)), Some(&Some(&Name::from("JKL"))));
    assert_eq!(entities.get(&Entity::from_raw(4)), Some(&None));
    assert_eq!(entities.get(&Entity::from_bits((2 << 32) + 5)), Some(&None));
}
