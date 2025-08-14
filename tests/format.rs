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

fn extract(world: &World, with_checkpoints: bool) -> Snapshot {
    let mut b = Snapshot::builder(world).extract_all_entities();

    if with_checkpoints {
        b = b.extract_resource::<Checkpoints>()
    }

    b.build()
}

const JSON_SNAPSHOT: &str = r#"{
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

const JSON_CHECKPOINTS_V3: &str = r#"{
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
    "resources": {},
    "rollbacks": {
        "checkpoints": [
            {
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
            }
        ],
        "active": 0
    }
}"#;

const JSON_CHECKPOINTS_V4: &str = r#"{
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
    "resources": {
        "bevy_save::Checkpoints": {
            "snapshots": [
                {
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
                }
            ],
            "active": 0
        }
    }
}"#;

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

    println!("{}", output);
    assert_eq!(output, JSON_SNAPSHOT);

    let deserializer = SnapshotDeserializer::new(&registry);

    let mut de = serde_json::Deserializer::from_str(&output);

    let value = deserializer.deserialize(&mut de).unwrap();

    let output = json_serialize(&value, &registry);

    assert_eq!(output, JSON_SNAPSHOT);
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

    assert_eq!(output, JSON_CHECKPOINTS_V4);

    let deserializer = SnapshotDeserializer::new(&registry);

    let mut de = serde_json::Deserializer::from_str(&output);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = json_serialize(&value, &registry);

    assert_eq!(output, JSON_CHECKPOINTS_V4);
}

#[test]
fn test_format_json_checkpoints_backcompat() {
    let (mut app, _) = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();
    let deserializer = SnapshotDeserializer::new(&registry).version(SnapshotVersion::V3);

    let mut de = serde_json::Deserializer::from_str(JSON_CHECKPOINTS_V3);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = json_serialize(&value, &registry);

    assert_eq!(output, JSON_CHECKPOINTS_V4);
}

const MP_SNAPSHOT: &[u8] = &[
    146, 133, 207, 0, 0, 0, 1, 0, 0, 0, 0, 145, 128, 207, 0, 0, 0, 1, 0, 0, 0, 1, 145, 131, 175,
    102, 111, 114, 109, 97, 116, 58, 58, 67, 111, 108, 108, 101, 99, 116, 145, 147, 3, 4, 5, 176,
    102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105, 116, 105, 111, 110, 147, 202, 0, 0, 0,
    0, 202, 63, 128, 0, 0, 202, 64, 0, 0, 0, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110,
    105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 2, 145, 131, 173, 102, 111, 114, 109, 97, 116, 58, 58,
    66, 97, 115, 105, 99, 145, 42, 176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108, 108, 97,
    98, 108, 101, 145, 77, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144, 207,
    0, 0, 0, 1, 0, 0, 0, 3, 145, 130, 176, 102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105,
    116, 105, 111, 110, 147, 202, 64, 192, 0, 0, 202, 64, 224, 0, 0, 202, 65, 0, 0, 0, 172, 102,
    111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 4, 145, 129,
    176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108, 108, 97, 98, 108, 101, 145, 192, 128,
];

const MP_CHECKPOINTS_V3: &[u8] = &[
    147, 133, 207, 0, 0, 0, 1, 0, 0, 0, 0, 145, 128, 207, 0, 0, 0, 1, 0, 0, 0, 1, 145, 131, 175,
    102, 111, 114, 109, 97, 116, 58, 58, 67, 111, 108, 108, 101, 99, 116, 145, 147, 3, 4, 5, 176,
    102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105, 116, 105, 111, 110, 147, 202, 0, 0, 0,
    0, 202, 63, 128, 0, 0, 202, 64, 0, 0, 0, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110,
    105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 2, 145, 131, 173, 102, 111, 114, 109, 97, 116, 58, 58,
    66, 97, 115, 105, 99, 145, 42, 176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108, 108, 97,
    98, 108, 101, 145, 77, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144, 207,
    0, 0, 0, 1, 0, 0, 0, 3, 145, 130, 176, 102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105,
    116, 105, 111, 110, 147, 202, 64, 192, 0, 0, 202, 64, 224, 0, 0, 202, 65, 0, 0, 0, 172, 102,
    111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 4, 145, 129,
    176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108, 108, 97, 98, 108, 101, 145, 192, 128,
    146, 145, 146, 133, 207, 0, 0, 0, 1, 0, 0, 0, 0, 145, 128, 207, 0, 0, 0, 1, 0, 0, 0, 1, 145,
    131, 175, 102, 111, 114, 109, 97, 116, 58, 58, 67, 111, 108, 108, 101, 99, 116, 145, 147, 3, 4,
    5, 176, 102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105, 116, 105, 111, 110, 147, 202,
    0, 0, 0, 0, 202, 63, 128, 0, 0, 202, 64, 0, 0, 0, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85,
    110, 105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 2, 145, 131, 173, 102, 111, 114, 109, 97, 116,
    58, 58, 66, 97, 115, 105, 99, 145, 42, 176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108,
    108, 97, 98, 108, 101, 145, 77, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116,
    144, 207, 0, 0, 0, 1, 0, 0, 0, 3, 145, 130, 176, 102, 111, 114, 109, 97, 116, 58, 58, 80, 111,
    115, 105, 116, 105, 111, 110, 147, 202, 64, 192, 0, 0, 202, 64, 224, 0, 0, 202, 65, 0, 0, 0,
    172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 4,
    145, 129, 176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108, 108, 97, 98, 108, 101, 145,
    192, 128, 0,
];

const MP_CHECKPOINTS_V4: &[u8] = &[
    146, 133, 207, 0, 0, 0, 1, 0, 0, 0, 0, 145, 128, 207, 0, 0, 0, 1, 0, 0, 0, 1, 145, 131, 175,
    102, 111, 114, 109, 97, 116, 58, 58, 67, 111, 108, 108, 101, 99, 116, 145, 147, 3, 4, 5, 176,
    102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105, 116, 105, 111, 110, 147, 202, 0, 0, 0,
    0, 202, 63, 128, 0, 0, 202, 64, 0, 0, 0, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110,
    105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 2, 145, 131, 173, 102, 111, 114, 109, 97, 116, 58, 58,
    66, 97, 115, 105, 99, 145, 42, 176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108, 108, 97,
    98, 108, 101, 145, 77, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144, 207,
    0, 0, 0, 1, 0, 0, 0, 3, 145, 130, 176, 102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105,
    116, 105, 111, 110, 147, 202, 64, 192, 0, 0, 202, 64, 224, 0, 0, 202, 65, 0, 0, 0, 172, 102,
    111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 4, 145, 129,
    176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108, 108, 97, 98, 108, 101, 145, 192, 129,
    182, 98, 101, 118, 121, 95, 115, 97, 118, 101, 58, 58, 67, 104, 101, 99, 107, 112, 111, 105,
    110, 116, 115, 146, 145, 146, 133, 207, 0, 0, 0, 1, 0, 0, 0, 0, 145, 128, 207, 0, 0, 0, 1, 0,
    0, 0, 1, 145, 131, 175, 102, 111, 114, 109, 97, 116, 58, 58, 67, 111, 108, 108, 101, 99, 116,
    145, 147, 3, 4, 5, 176, 102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105, 116, 105, 111,
    110, 147, 202, 0, 0, 0, 0, 202, 63, 128, 0, 0, 202, 64, 0, 0, 0, 172, 102, 111, 114, 109, 97,
    116, 58, 58, 85, 110, 105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 2, 145, 131, 173, 102, 111, 114,
    109, 97, 116, 58, 58, 66, 97, 115, 105, 99, 145, 42, 176, 102, 111, 114, 109, 97, 116, 58, 58,
    78, 117, 108, 108, 97, 98, 108, 101, 145, 77, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85,
    110, 105, 116, 144, 207, 0, 0, 0, 1, 0, 0, 0, 3, 145, 130, 176, 102, 111, 114, 109, 97, 116,
    58, 58, 80, 111, 115, 105, 116, 105, 111, 110, 147, 202, 64, 192, 0, 0, 202, 64, 224, 0, 0,
    202, 65, 0, 0, 0, 172, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 144, 207, 0, 0,
    0, 1, 0, 0, 0, 4, 145, 129, 176, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108, 108, 97,
    98, 108, 101, 145, 192, 128, 0,
];

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

    assert_eq!(output, MP_SNAPSHOT);

    let deserializer = SnapshotDeserializer::new(&registry);

    let mut de = rmp_serde::Deserializer::new(&*output);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = mp_serialize(&value, &registry);

    assert_eq!(output, MP_SNAPSHOT);
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

    assert_eq!(output, MP_CHECKPOINTS_V4);

    let deserializer = SnapshotDeserializer::new(&registry);

    let mut de = rmp_serde::Deserializer::new(&*output);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = mp_serialize(&value, &registry);

    assert_eq!(output, MP_CHECKPOINTS_V4);
}

#[test]
fn test_format_mp_checkpoints_backcompat() {
    let (mut app, _) = init_app();
    let world = app.world_mut();

    let registry = world.resource::<AppTypeRegistry>().read();
    let deserializer = SnapshotDeserializer::new(&registry).version(SnapshotVersion::V3);

    let mut de = rmp_serde::Deserializer::new(MP_CHECKPOINTS_V3);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = mp_serialize(&value, &registry);

    assert_eq!(output, MP_CHECKPOINTS_V4);
}

const POSTCARD_SNAPSHOT: &[u8] = &[
    5, 128, 128, 128, 128, 16, 0, 129, 128, 128, 128, 16, 3, 15, 102, 111, 114, 109, 97, 116, 58,
    58, 67, 111, 108, 108, 101, 99, 116, 3, 3, 4, 5, 16, 102, 111, 114, 109, 97, 116, 58, 58, 80,
    111, 115, 105, 116, 105, 111, 110, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 12, 102, 111, 114,
    109, 97, 116, 58, 58, 85, 110, 105, 116, 130, 128, 128, 128, 16, 3, 13, 102, 111, 114, 109, 97,
    116, 58, 58, 66, 97, 115, 105, 99, 42, 16, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108,
    108, 97, 98, 108, 101, 1, 77, 12, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 131,
    128, 128, 128, 16, 2, 16, 102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105, 116, 105,
    111, 110, 0, 0, 192, 64, 0, 0, 224, 64, 0, 0, 0, 65, 12, 102, 111, 114, 109, 97, 116, 58, 58,
    85, 110, 105, 116, 132, 128, 128, 128, 16, 1, 16, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117,
    108, 108, 97, 98, 108, 101, 0, 0,
];

const POSTCARD_CHECKPOINTS: &[u8] = &[
    5, 128, 128, 128, 128, 16, 0, 129, 128, 128, 128, 16, 3, 15, 102, 111, 114, 109, 97, 116, 58,
    58, 67, 111, 108, 108, 101, 99, 116, 3, 3, 4, 5, 16, 102, 111, 114, 109, 97, 116, 58, 58, 80,
    111, 115, 105, 116, 105, 111, 110, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 12, 102, 111, 114,
    109, 97, 116, 58, 58, 85, 110, 105, 116, 130, 128, 128, 128, 16, 3, 13, 102, 111, 114, 109, 97,
    116, 58, 58, 66, 97, 115, 105, 99, 42, 16, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108,
    108, 97, 98, 108, 101, 1, 77, 12, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 131,
    128, 128, 128, 16, 2, 16, 102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105, 116, 105,
    111, 110, 0, 0, 192, 64, 0, 0, 224, 64, 0, 0, 0, 65, 12, 102, 111, 114, 109, 97, 116, 58, 58,
    85, 110, 105, 116, 132, 128, 128, 128, 16, 1, 16, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117,
    108, 108, 97, 98, 108, 101, 0, 1, 22, 98, 101, 118, 121, 95, 115, 97, 118, 101, 58, 58, 67,
    104, 101, 99, 107, 112, 111, 105, 110, 116, 115, 1, 5, 128, 128, 128, 128, 16, 0, 129, 128,
    128, 128, 16, 3, 15, 102, 111, 114, 109, 97, 116, 58, 58, 67, 111, 108, 108, 101, 99, 116, 3,
    3, 4, 5, 16, 102, 111, 114, 109, 97, 116, 58, 58, 80, 111, 115, 105, 116, 105, 111, 110, 0, 0,
    0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 12, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116,
    130, 128, 128, 128, 16, 3, 13, 102, 111, 114, 109, 97, 116, 58, 58, 66, 97, 115, 105, 99, 42,
    16, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108, 108, 97, 98, 108, 101, 1, 77, 12, 102,
    111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 131, 128, 128, 128, 16, 2, 16, 102, 111,
    114, 109, 97, 116, 58, 58, 80, 111, 115, 105, 116, 105, 111, 110, 0, 0, 192, 64, 0, 0, 224, 64,
    0, 0, 0, 65, 12, 102, 111, 114, 109, 97, 116, 58, 58, 85, 110, 105, 116, 132, 128, 128, 128,
    16, 1, 16, 102, 111, 114, 109, 97, 116, 58, 58, 78, 117, 108, 108, 97, 98, 108, 101, 0, 0, 1,
    0,
];

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

    assert_eq!(output, POSTCARD_SNAPSHOT);
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

    assert_eq!(output, POSTCARD_CHECKPOINTS);

    let deserializer = SnapshotDeserializer::new(&registry);

    let mut de = postcard::Deserializer::from_bytes(&output);
    let value = deserializer.deserialize(&mut de).unwrap();
    let output = postcard_serialize(&value, &registry);

    assert_eq!(output, POSTCARD_CHECKPOINTS);
}
