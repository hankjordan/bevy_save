use bevy::prelude::*;
use bevy_save::{
    dynamic::{
        DynamicSnapshotDeserializer,
        DynamicSnapshotSerializer,
    },
    prelude::*,
};
use serde::{
    de::DeserializeSeed,
    Serialize,
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

fn init_app() -> App {
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

    let world = &mut app.world;

    world.spawn(());

    world.spawn((
        Position {
            x: 0.0,
            y: 1.0,
            z: 2.0,
        },
        Collect {
            data: vec![3, 4, 5],
        },
        Unit,
    ));

    world.spawn((Basic { data: 42 }, Nullable { data: Some(77) }, Unit));

    world.spawn((
        Position {
            x: 6.0,
            y: 7.0,
            z: 8.0,
        },
        Unit,
    ));

    world.spawn(Nullable { data: None });

    app
}

fn extract(world: &World) -> DynamicSnapshot {
    DynamicSnapshot::builder(world)
        .extract_all_entities()
        .build()
}

#[test]
fn test_json() {
    fn serialize(snapshot: &DynamicSnapshot, registry: &AppTypeRegistry) -> String {
        let serializer = DynamicSnapshotSerializer { snapshot, registry };

        let mut buf = Vec::new();
        let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);

        serializer.serialize(&mut ser).unwrap();

        String::from_utf8(buf).unwrap()
    }

    let mut app = init_app();
    let world = &mut app.world;

    let registry = world.resource::<AppTypeRegistry>();
    let snapshot = extract(world);

    let output = serialize(&snapshot, registry);
    let expected = r#"{
    "entities": {
        "0": {
            "components": {}
        },
        "1": {
            "components": {
                "format::Position": {
                    "x": 0.0,
                    "y": 1.0,
                    "z": 2.0
                },
                "format::Collect": {
                    "data": [
                        3,
                        4,
                        5
                    ]
                },
                "format::Unit": {}
            }
        },
        "2": {
            "components": {
                "format::Unit": {},
                "format::Basic": {
                    "data": 42
                },
                "format::Nullable": {
                    "data": 77
                }
            }
        },
        "3": {
            "components": {
                "format::Position": {
                    "x": 6.0,
                    "y": 7.0,
                    "z": 8.0
                },
                "format::Unit": {}
            }
        },
        "4": {
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

    let deserializer = DynamicSnapshotDeserializer {
        registry: &registry.read(),
    };

    let mut de = serde_json::Deserializer::from_str(&output);

    let value = deserializer.deserialize(&mut de).unwrap();

    let output = serialize(&value, registry);

    assert_eq!(output, expected);
}

#[test]
fn test_mp() {
    fn serialize(snapshot: &DynamicSnapshot, registry: &AppTypeRegistry) -> Vec<u8> {
        let serializer = DynamicSnapshotSerializer { snapshot, registry };

        let mut buf = Vec::new();
        let mut ser = rmp_serde::Serializer::new(&mut buf);

        serializer.serialize(&mut ser).unwrap();

        buf
    }

    let mut app = init_app();
    let world = &mut app.world;

    let registry = world.resource::<AppTypeRegistry>();
    let snapshot = extract(world);

    let output = serialize(&snapshot, registry);
    let expected = [
        146, 133, 0, 145, 128, 1, 145, 131, 176, 102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115,
        105, 116, 105, 111, 110, 147, 202, 0, 0, 0, 0, 202, 63, 128, 0, 0, 202, 64, 0, 0, 0, 175,
        102, 111, 114, 109, 97, 116, 58, 58, 67, 111, 108, 108, 101, 99, 116, 145, 147, 3, 4, 5,
        172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144, 2, 145, 131, 172, 102,
        111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144, 173, 102, 111, 114, 109, 97, 116,
        58, 58, 66, 97, 115, 105, 99, 145, 42, 176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117,
        108, 108, 97, 98, 108, 101, 145, 77, 3, 145, 130, 176, 102, 111, 114, 109, 97, 116, 58, 58,
        80, 111, 115, 105, 116, 105, 111, 110, 147, 202, 64, 192, 0, 0, 202, 64, 224, 0, 0, 202,
        65, 0, 0, 0, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144, 4, 145, 129,
        176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108, 108, 97, 98, 108, 101, 145, 192,
        128,
    ];

    assert_eq!(output, expected);

    let deserializer = DynamicSnapshotDeserializer {
        registry: &registry.read(),
    };

    let mut de = rmp_serde::Deserializer::new(&*output);

    let value = deserializer.deserialize(&mut de).unwrap();

    let output = serialize(&value, registry);

    assert_eq!(output, expected);
}
