use bevy::{
    prelude::*,
    reflect::TypeRegistry,
};
use bevy_save::{
    prelude::*,
    reflect::{
        SnapshotDeserializer,
        SnapshotSerializer,
        SnapshotVersion,
        checkpoint::Checkpoints,
    },
};
use serde::{
    Serialize,
    de::DeserializeSeed,
};

use crate::data::{
    json,
    mp,
    pc,
};

mod data;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Unit;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Basic {
    data: Entity,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Collect {
    data: Vec<Entity>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Nullable {
    data: Option<Entity>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

fn empty_app() -> App {
    let mut app = App::new();

    app //
        .add_plugins((MinimalPlugins, SavePlugins))
        .register_type::<Unit>()
        .register_type::<Basic>()
        .register_type::<Collect>()
        .register_type::<Vec<u32>>()
        .register_type::<Nullable>()
        .register_type::<Option<u32>>()
        .register_type::<Position>();

    app
}

fn init_app() -> (App, Vec<Entity>) {
    let mut app = empty_app();
    let world = app.world_mut();

    let ids = vec![
        world.spawn(()).id(),
        world
            .spawn((
                Position {
                    x: 0.0,
                    y: 1.0,
                    z: 2.0,
                },
                Collect {
                    data: vec![
                        Entity::from_raw(3),
                        Entity::from_raw(4),
                        Entity::from_raw(5),
                    ],
                },
                Unit,
            ))
            .id(),
        world
            .spawn((
                Basic {
                    data: Entity::from_raw(42),
                },
                Nullable {
                    data: Some(Entity::from_raw(77)),
                },
                Unit,
            ))
            .id(),
        world
            .spawn((
                Position {
                    x: 6.0,
                    y: 7.0,
                    z: 8.0,
                },
                Unit,
            ))
            .id(),
        world.spawn(Nullable { data: None }).id(),
    ];

    (app, ids)
}

fn extract(world: &World, with_checkpoints: bool) -> Snapshot {
    let mut b = Snapshot::builder(world).extract_all_entities();

    if with_checkpoints {
        b = b.extract_resource::<Checkpoints>()
    }

    b.build()
}

fn json_serialize(snapshot: &Snapshot, registry: &TypeRegistry) -> String {
    let serializer = SnapshotSerializer::new(snapshot, registry);

    let mut buf = Vec::new();
    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);

    serializer.serialize(&mut ser).unwrap();

    String::from_utf8(buf).unwrap()
}

#[test]
fn test_format_json() {
    let (mut app, _) = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();
    let snapshot = extract(world, false);

    let output = json_serialize(&snapshot, &registry);

    println!("JSON_SNAPSHOT: {}", output);
    assert_eq!(output, json::SNAPSHOT_V4);

    let deserializer = SnapshotDeserializer::new(&registry);

    let mut de = serde_json::Deserializer::from_str(&output);

    let value = deserializer.deserialize(&mut de).unwrap();

    let output = json_serialize(&value, &registry);

    assert_eq!(output, json::SNAPSHOT_V4);
}

#[test]
fn test_format_json_checkpoints() {
    let (mut app, _) = init_app();
    let world = app.world_mut();

    let snapshot = extract(world, false);

    world.resource_mut::<Checkpoints>().checkpoint(snapshot);

    let registry = world.resource::<AppTypeRegistry>().read();

    let snapshot = extract(world, true);
    let output = json_serialize(&snapshot, &registry);

    println!("JSON_CHECKPOINTS_V4: {}", output);
    assert_eq!(output, json::CHECKPOINTS_V4);

    let deserializer = SnapshotDeserializer::new(&registry);

    let mut de = serde_json::Deserializer::from_str(&output);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = json_serialize(&value, &registry);

    assert_eq!(output, json::CHECKPOINTS_V4);
}

#[test]
fn test_format_json_checkpoints_backcompat() {
    let (mut app, _) = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();
    let deserializer = SnapshotDeserializer::new(&registry).version(SnapshotVersion::V3);

    let mut de = serde_json::Deserializer::from_str(json::CHECKPOINTS_V3);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = json_serialize(&value, &registry);

    assert_eq!(output, json::CHECKPOINTS_V4);
}

fn mp_serialize(snapshot: &Snapshot, registry: &TypeRegistry) -> Vec<u8> {
    let serializer = SnapshotSerializer::new(snapshot, registry);

    let mut buf = Vec::new();
    let mut ser = rmp_serde::Serializer::new(&mut buf);

    serializer.serialize(&mut ser).unwrap();

    buf
}

#[test]
fn test_format_mp() {
    let (mut app, _) = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();
    let snapshot = extract(world, false);

    let output = mp_serialize(&snapshot, &registry);

    assert_eq!(output, mp::SNAPSHOT_V4);

    let deserializer = SnapshotDeserializer::new(&registry);

    let mut de = rmp_serde::Deserializer::new(&*output);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = mp_serialize(&value, &registry);

    assert_eq!(output, mp::SNAPSHOT_V4);
}

#[test]
fn test_format_mp_checkpoints() {
    let (mut app, _) = init_app();
    let world = app.world_mut();

    let snapshot = extract(world, false);

    world.resource_mut::<Checkpoints>().checkpoint(snapshot);

    let registry = world.resource::<AppTypeRegistry>().read();
    let snapshot = extract(world, true);

    let output = mp_serialize(&snapshot, &registry);

    assert_eq!(output, mp::CHECKPOINTS_V4);

    let deserializer = SnapshotDeserializer::new(&registry);

    let mut de = rmp_serde::Deserializer::new(&*output);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = mp_serialize(&value, &registry);

    assert_eq!(output, mp::CHECKPOINTS_V4);
}

#[test]
fn test_format_mp_checkpoints_backcompat() {
    let (mut app, _) = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();
    let deserializer = SnapshotDeserializer::new(&registry).version(SnapshotVersion::V3);

    let mut de = rmp_serde::Deserializer::new(mp::CHECKPOINTS_V3);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = mp_serialize(&value, &registry);

    assert_eq!(output, mp::CHECKPOINTS_V4);
}

fn postcard_serialize(snapshot: &Snapshot, registry: &TypeRegistry) -> Vec<u8> {
    postcard::to_stdvec(&SnapshotSerializer::new(snapshot, registry)).unwrap()
}

#[test]
fn test_format_postcard() {
    let (mut app, _) = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();
    let snapshot = extract(world, false);

    let output = postcard_serialize(&snapshot, &registry);

    assert_eq!(output, pc::SNAPSHOT_V4);
}

#[test]
fn test_format_postcard_checkpoints() {
    let (mut app, _) = init_app();
    let world = app.world_mut();

    let snapshot = extract(world, false);

    world.resource_mut::<Checkpoints>().checkpoint(snapshot);

    let registry = world.resource::<AppTypeRegistry>().read();
    let snapshot = extract(world, true);

    let output = postcard_serialize(&snapshot, &registry);

    assert_eq!(output, pc::CHECKPOINTS_V4);

    let deserializer = SnapshotDeserializer::new(&registry);

    let mut de = postcard::Deserializer::from_bytes(&output);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = postcard_serialize(&value, &registry);

    assert_eq!(output, pc::CHECKPOINTS_V4);
}
