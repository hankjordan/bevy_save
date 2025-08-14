use bevy::{
    prelude::*,
    reflect::GetTypeRegistration,
};
use bevy_save::{
    prelude::*,
    reflect::{
        ReflectMap,
        serde::{
            ReflectMapDeserializer,
            ReflectMapSerializer,
        },
    },
};
use serde::de::DeserializeSeed;

#[derive(Reflect)]
struct Pos {
    x: f32,
    y: f32,
}

#[derive(Reflect)]
struct Pos2 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Reflect, Component, Debug, PartialEq)]
#[reflect(Component, Migrate)]
struct Position {
    xyz: (f32, f32, f32),
}

impl Migrate for Position {
    fn migrator() -> Migrator<Self> {
        #[derive(Reflect)]
        #[type_path = "migrate"]
        #[type_name = "Pos"]
        struct PosV0_1 {
            x: f32,
            y: f32,
        }

        Migrator::new::<PosV0_1>("0.1.0")
            .version("0.2.0", |v1| {
                #[derive(Reflect)]
                #[type_path = "migrate"]
                #[type_name = "Position"]
                struct PosV0_2 {
                    x: f32,
                    y: f32,
                }

                Some(PosV0_2 { x: v1.x, y: v1.y })
            })
            .version("0.3.0", |v2| {
                #[derive(Reflect)]
                #[type_path = "migrate"]
                #[type_name = "Position"]
                struct PosV0_3 {
                    x: f32,
                    y: f32,
                    z: f32,
                }

                Some(PosV0_3 {
                    x: v2.x,
                    y: v2.y,
                    z: 0.0,
                })
            })
            .version("0.4.0", |v2| {
                Some(Self {
                    xyz: (v2.x, v2.y, v2.z),
                })
            })
    }
}

fn init_app() -> App {
    let mut app = App::new();

    app.add_plugins(SavePlugins).register_type::<Position>();

    let world = app.world_mut();

    world.spawn(Position {
        xyz: (0.0, 1.0, 0.0),
    });
    world.spawn(Position {
        xyz: (2.0, 3.0, 0.0),
    });
    world.spawn(Position {
        xyz: (4.0, 5.0, 6.0),
    });

    app
}

fn json_serialize<T: serde::Serialize>(value: &T) -> Result<String, anyhow::Error> {
    let mut buf = Vec::new();
    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);
    value.serialize(&mut ser)?;
    Ok(String::from_utf8(buf)?)
}

#[test]
fn test_migrate() {
    let mut app = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();

    let migrator = registry
        .get(Position::get_type_registration().type_id())
        .and_then(|r| r.data::<ReflectMigrate>())
        .expect("Invalid type registration");

    let out = migrator
        .migrate(&Pos { x: 0.0, y: 1.0 }, "0.1.0")
        .and_then(|r| r.take().ok());

    println!("{:?}", out);
    assert_eq!(
        out,
        Some(Position {
            xyz: (0.0, 1.0, 0.0)
        })
    );

    let out = migrator
        .migrate(
            &Pos2 {
                x: 2.0,
                y: 3.0,
                z: 4.0,
            },
            "0.3.0",
        )
        .and_then(|r| r.take().ok());

    println!("{:?}", out);
    assert_eq!(
        out,
        Some(Position {
            xyz: (2.0, 3.0, 4.0)
        })
    );
}

const JSON_REFLECT_MAP: &str = r#"{
    "migrate::Position 0.4.0": {
        "xyz": [
            0.0,
            1.0,
            2.0
        ]
    },
    "migrate::Position 0.4.0": {
        "xyz": [
            2.0,
            3.0,
            4.0
        ]
    }
}"#;

const JSON_REFLECT_MAP_OLD: &str = r#"{
    "migrate::Pos 0.1.0": {
        "x": -2.0,
        "y": -1.0
    },
    "migrate::Position 0.2.0": {
        "x": 0.0,
        "y": 1.0
    },
    "migrate::Position 0.3.0": {
        "x": 3.0,
        "y": 4.0,
        "z": 5.0
    },
    "migrate::Position 0.4.0": {
        "xyz": [
            6.0,
            7.0,
            8.0
        ]
    }
}"#;

const JSON_SNAPSHOT: &str = r#"{
    "entities": {
        "4294967296": {
            "components": {
                "migrate::Position 0.4.0": {
                    "xyz": [
                        0.0,
                        1.0,
                        0.0
                    ]
                }
            }
        },
        "4294967297": {
            "components": {
                "migrate::Position 0.4.0": {
                    "xyz": [
                        2.0,
                        3.0,
                        0.0
                    ]
                }
            }
        },
        "4294967298": {
            "components": {
                "migrate::Position 0.4.0": {
                    "xyz": [
                        4.0,
                        5.0,
                        6.0
                    ]
                }
            }
        }
    },
    "resources": {
        "bevy_save::Checkpoints": {
            "snapshots": [],
            "active": null
        }
    }
}"#;

const JSON_SNAPSHOT_OLD: &str = r#"{
    "entities": {
        "4294967296": {
            "components": {
                "migrate::Pos 0.1.0": {
                    "x": 0.0,
                    "y": 1.0
                }
            }
        },
        "4294967297": {
            "components": {
                "migrate::Position 0.2.0": {
                    "x": 2.0,
                    "y": 3.0
                }
            }
        },
        "4294967298": {
            "components": {
                "migrate::Position 0.3.0": {
                    "x": 4.0,
                    "y": 5.0,
                    "z": 6.0
                }
            }
        }
    },
    "resources": {
        "bevy_save::Checkpoints": {
            "snapshots": [],
            "active": null
        }
    }
}"#;

#[test]
fn test_migrate_serialize() {
    let mut app = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();

    let entries = ReflectMap(vec![
        Box::new(Position {
            xyz: (0.0, 1.0, 2.0),
        })
        .into_partial_reflect()
        .into(),
        Box::new(Position {
            xyz: (2.0, 3.0, 4.0),
        })
        .into_partial_reflect()
        .into(),
    ]);
    let ser = ReflectMapSerializer::new(&entries, &registry);

    let out = json_serialize(&ser).expect("Failed to serialize");

    println!("{}", out);
    assert_eq!(out, JSON_REFLECT_MAP);
}

#[test]
fn test_migrate_deserialize() {
    let mut app = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();

    let seed = ReflectMapDeserializer::new(&registry);

    let mut de = serde_json::Deserializer::from_str(JSON_REFLECT_MAP_OLD);
    let out = seed.deserialize(&mut de).unwrap();

    println!("{:?}", out);

    let out = out
        .iter()
        .map(|r| Position::from_reflect(r).expect("Invalid reflect"))
        .collect::<Vec<_>>();

    println!("{:?}", out);
    assert_eq!(out, vec![
        Position {
            xyz: (-2.0, -1.0, 0.0)
        },
        Position {
            xyz: (0.0, 1.0, 0.0)
        },
        Position {
            xyz: (3.0, 4.0, 5.0)
        },
        Position {
            xyz: (6.0, 7.0, 8.0)
        }
    ]);
}

#[test]
fn test_migrate_snapshot() {
    let mut app = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();
    let snapshot = Snapshot::from_world(world);

    let out = json_serialize(&snapshot.serializer(&registry)).expect("Failed to serialize");

    println!("{}", out);
    assert_eq!(out, JSON_SNAPSHOT);

    let deserializer = Snapshot::deserializer(&registry);
    let mut de = serde_json::Deserializer::from_str(&out);
    let snapshot = deserializer.deserialize(&mut de).unwrap();

    let out = json_serialize(&snapshot.serializer(&registry)).expect("Failed to serialize");

    println!("{}", out);
    assert_eq!(out, JSON_SNAPSHOT);

    let deserializer = Snapshot::deserializer(&registry);
    let mut de = serde_json::Deserializer::from_str(JSON_SNAPSHOT_OLD);
    let snapshot = deserializer.deserialize(&mut de).unwrap();

    let out = json_serialize(&snapshot.serializer(&registry)).expect("Failed to serialize");

    println!("{}", out);
    assert_eq!(out, JSON_SNAPSHOT);
}
