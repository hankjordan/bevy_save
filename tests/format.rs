use bevy::{
    prelude::*,
    reflect::TypeRegistry,
};
use bevy_save::{
    prelude::*,
    reflect::{
        SnapshotDeserializer,
        SnapshotSerializer,
    },
};
use serde::{
    Serialize,
    de::DeserializeSeed,
};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Unit;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Basic {
    data: u32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Collect {
    data: Vec<u32>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Nullable {
    data: Option<u32>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

fn init_app() -> (App, Vec<Entity>) {
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
                    data: vec![3, 4, 5],
                },
                Unit,
            ))
            .id(),
        world
            .spawn((Basic { data: 42 }, Nullable { data: Some(77) }, Unit))
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

fn extract(world: &World) -> Snapshot {
    Snapshot::builder(world).extract_all_entities().build()
}

#[test]
fn test_json() {
    fn serialize(snapshot: &Snapshot, registry: &TypeRegistry) -> String {
        let serializer = SnapshotSerializer { snapshot, registry };

        let mut buf = Vec::new();
        let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);

        serializer.serialize(&mut ser).unwrap();

        String::from_utf8(buf).unwrap()
    }

    let (mut app, _) = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();
    let snapshot = extract(world);

    let output = serialize(&snapshot, &registry);
    let expected = r#"{
    "entities": {
        "4294967296": {
            "components": {}
        },
        "4294967297": {
            "components": {
                "format::Collect": {
                    "data": [
                        3,
                        4,
                        5
                    ]
                },
                "format::Position": {
                    "x": 0.0,
                    "y": 1.0,
                    "z": 2.0
                },
                "format::Unit": {}
            }
        },
        "4294967298": {
            "components": {
                "format::Basic": {
                    "data": 42
                },
                "format::Nullable": {
                    "data": 77
                },
                "format::Unit": {}
            }
        },
        "4294967299": {
            "components": {
                "format::Position": {
                    "x": 6.0,
                    "y": 7.0,
                    "z": 8.0
                },
                "format::Unit": {}
            }
        },
        "4294967300": {
            "components": {
                "format::Nullable": {
                    "data": null
                }
            }
        }
    },
    "resources": {}
}"#;

    assert_eq!(output, expected);

    let deserializer = SnapshotDeserializer {
        registry: &registry,
    };

    let mut de = serde_json::Deserializer::from_str(&output);

    let value = deserializer.deserialize(&mut de).unwrap();

    let output = serialize(&value, &registry);

    assert_eq!(output, expected);
}

#[test]
fn test_mp() {
    fn serialize(snapshot: &Snapshot, registry: &TypeRegistry) -> Vec<u8> {
        let serializer = SnapshotSerializer { snapshot, registry };

        let mut buf = Vec::new();
        let mut ser = rmp_serde::Serializer::new(&mut buf);

        serializer.serialize(&mut ser).unwrap();

        buf
    }

    let (mut app, _) = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();
    let snapshot = extract(world);

    let output = serialize(&snapshot, &registry);
    let expected = vec![
        146, 133, 207, 0, 0, 0, 1, 0, 0, 0, 0, 145, 128, 207, 0, 0, 0, 1, 0, 0, 0, 1, 145, 131,
        175, 102, 111, 114, 109, 97, 116, 58, 58, 67, 111, 108, 108, 101, 99, 116, 145, 147, 3, 4,
        5, 176, 102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105, 116, 105, 111, 110, 147,
        202, 0, 0, 0, 0, 202, 63, 128, 0, 0, 202, 64, 0, 0, 0, 172, 102, 111, 114, 109, 97, 116,
        58, 58, 85, 110, 105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 2, 145, 131, 173, 102, 111, 114,
        109, 97, 116, 58, 58, 66, 97, 115, 105, 99, 145, 42, 176, 102, 111, 114, 109, 97, 116, 58,
        58, 78, 117, 108, 108, 97, 98, 108, 101, 145, 77, 172, 102, 111, 114, 109, 97, 116, 58, 58,
        85, 110, 105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 3, 145, 130, 176, 102, 111, 114, 109, 97,
        116, 58, 58, 80, 111, 115, 105, 116, 105, 111, 110, 147, 202, 64, 192, 0, 0, 202, 64, 224,
        0, 0, 202, 65, 0, 0, 0, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144,
        207, 0, 0, 0, 1, 0, 0, 0, 4, 145, 129, 176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117,
        108, 108, 97, 98, 108, 101, 145, 192, 128,
    ];

    assert_eq!(output, expected);

    let deserializer = SnapshotDeserializer {
        registry: &registry,
    };

    let mut de = rmp_serde::Deserializer::new(&*output);

    let value = deserializer.deserialize(&mut de).unwrap();

    let output = serialize(&value, &registry);

    assert_eq!(output, expected);
}
