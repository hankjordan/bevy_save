use bevy::{
    prelude::*,
    reflect::serde::TypedReflectSerializer,
};
use bevy_save::{
    clone_reflect_value,
    prelude::*,
    reflect::{
        DynamicValue,
        ReflectMap,
        checkpoint::Checkpoints,
    },
};

fn json_serialize<T: serde::Serialize>(value: &T) -> Result<String, anyhow::Error> {
    let mut buf = Vec::new();
    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);
    value.serialize(&mut ser)?;
    Ok(String::from_utf8(buf)?)
}

const REFLECT_JSON: &str = r#"{
    "objects": {
        "glam::Vec3": [
            0.0,
            1.0,
            2.0
        ],
        "reflect::Project": {
            "objects": {
                "glam::Vec3": [
                    3.0,
                    4.0,
                    5.0
                ]
            }
        },
        "reflect::Project": {
            "objects": {
                "reflect::Project": {
                    "objects": {
                        "glam::Vec3": [
                            6.0,
                            7.0,
                            8.0
                        ]
                    }
                }
            }
        }
    }
}"#;

#[test]
fn test_reflect() {
    #[derive(Reflect)]
    struct Project {
        objects: ReflectMap,
    }

    let mut app = App::new();

    let app = app.register_type::<Vec3>().register_type::<Project>();

    let registry = app.world().resource::<AppTypeRegistry>().read();

    let data = Project {
        objects: [
            Box::new(Vec3 {
                x: 0.0,
                y: 1.0,
                z: 2.0,
            })
            .into_partial_reflect(),
            Box::new(Project {
                objects: [Box::new(Vec3 {
                    x: 3.0,
                    y: 4.0,
                    z: 5.0,
                })
                .into_partial_reflect()]
                .into_iter()
                .collect(),
            })
            .into_partial_reflect(),
            Box::new(Project {
                objects: [Box::new(Project {
                    objects: [Box::new(Vec3 {
                        x: 6.0,
                        y: 7.0,
                        z: 8.0,
                    })
                    .into_partial_reflect()]
                    .into_iter()
                    .collect(),
                })
                .into_partial_reflect()]
                .into_iter()
                .collect(),
            })
            .into_partial_reflect(),
        ]
        .into_iter()
        .collect(),
    };
    let ser = TypedReflectSerializer::new(&data, &registry);
    let out = json_serialize(&ser).expect("Failed to serialize");

    println!("{}", out);

    assert_eq!(out, REFLECT_JSON);
}

const CLONE_JSON: &str = r#"[
    0.0,
    1.0,
    2.0
]"#;

const CLONE_LIST_JSON: &str = r#"{
    "glam::Vec3": [
        0.0,
        1.0,
        2.0
    ]
}"#;

#[test]
fn test_reflect_clone() {
    let mut app = App::new();

    let app = app
        .add_plugins(SavePlugins)
        .register_type::<Vec3>()
        .register_type::<DynamicValue>()
        .register_type::<ReflectMap>();

    let registry = app.world().resource::<AppTypeRegistry>().read();

    let data = Box::new(Vec3 {
        x: 0.0,
        y: 1.0,
        z: 2.0,
    })
    .into_partial_reflect();
    let cloned = clone_reflect_value(&*data, &registry);

    let ser = TypedReflectSerializer::new(&*data, &registry);
    let a = json_serialize(&ser).expect("Failed to serialize");

    let ser = TypedReflectSerializer::new(&*cloned, &registry);
    let b = json_serialize(&ser).expect("Failed to serialize");

    assert_eq!(a, CLONE_JSON);
    assert_eq!(b, CLONE_JSON);

    let data: ReflectMap = vec![
        Box::new(Vec3 {
            x: 0.0,
            y: 1.0,
            z: 2.0,
        })
        .into_partial_reflect(),
    ]
    .into();
    let cloned = clone_reflect_value(&data, &registry);

    let ser = TypedReflectSerializer::new(&data, &registry);
    let a = json_serialize(&ser).expect("Failed to serialize");

    let ser = TypedReflectSerializer::new(&*cloned, &registry);
    let b = json_serialize(&ser).expect("Failed to serialize");

    assert_eq!(a, CLONE_LIST_JSON);
    assert_eq!(b, CLONE_LIST_JSON);
}

const CHECKPOINT_JSON: &str = r#"{
    "entities": {
        "4294967296": {
            "components": {
                "bevy_transform::components::transform::Transform": {
                    "translation": [
                        0.0,
                        1.0,
                        2.0
                    ],
                    "rotation": [
                        0.0,
                        0.0,
                        0.0,
                        1.0
                    ],
                    "scale": [
                        1.0,
                        1.0,
                        1.0
                    ]
                }
            }
        }
    },
    "resources": {}
}"#;

const CHECKPOINT_LIST_JSON: &str = r#"{
    "items": [
        {
            "entities": {
                "4294967296": {
                    "components": {
                        "bevy_transform::components::transform::Transform": {
                            "translation": [
                                0.0,
                                1.0,
                                2.0
                            ],
                            "rotation": [
                                0.0,
                                0.0,
                                0.0,
                                1.0
                            ],
                            "scale": [
                                1.0,
                                1.0,
                                1.0
                            ]
                        }
                    }
                }
            },
            "resources": {}
        }
    ]
}"#;

const CHECKPOINT_LIST_NESTED_JSON: &str = r#"{
    "entities": {
        "4294967296": {
            "components": {
                "bevy_transform::components::transform::Transform": {
                    "translation": [
                        0.0,
                        1.0,
                        2.0
                    ],
                    "rotation": [
                        0.0,
                        0.0,
                        0.0,
                        1.0
                    ],
                    "scale": [
                        1.0,
                        1.0,
                        1.0
                    ]
                }
            }
        }
    },
    "resources": {
        "reflect::CheckpointList": {
            "items": [
                {
                    "entities": {
                        "4294967296": {
                            "components": {
                                "bevy_transform::components::transform::Transform": {
                                    "translation": [
                                        0.0,
                                        1.0,
                                        2.0
                                    ],
                                    "rotation": [
                                        0.0,
                                        0.0,
                                        0.0,
                                        1.0
                                    ],
                                    "scale": [
                                        1.0,
                                        1.0,
                                        1.0
                                    ]
                                }
                            }
                        }
                    },
                    "resources": {}
                }
            ]
        }
    }
}"#;

const CHECKPOINT_SNAPSHOT_JSON: &str = r#"{
    "entities": {},
    "resources": {
        "bevy_save::Checkpoints": {
            "snapshots": [
                {
                    "entities": {
                        "4294967296": {
                            "components": {
                                "bevy_transform::components::transform::Transform": {
                                    "translation": [
                                        0.0,
                                        1.0,
                                        2.0
                                    ],
                                    "rotation": [
                                        0.0,
                                        0.0,
                                        0.0,
                                        1.0
                                    ],
                                    "scale": [
                                        1.0,
                                        1.0,
                                        1.0
                                    ]
                                }
                            }
                        }
                    },
                    "resources": {}
                }
            ],
            "active": 0
        },
        "reflect::CheckpointList": {
            "items": [
                {
                    "entities": {
                        "4294967296": {
                            "components": {
                                "bevy_transform::components::transform::Transform": {
                                    "translation": [
                                        0.0,
                                        1.0,
                                        2.0
                                    ],
                                    "rotation": [
                                        0.0,
                                        0.0,
                                        0.0,
                                        1.0
                                    ],
                                    "scale": [
                                        1.0,
                                        1.0,
                                        1.0
                                    ]
                                }
                            }
                        }
                    },
                    "resources": {}
                }
            ]
        }
    }
}"#;

#[test]
fn test_reflect_checkpoints() {
    let mut app = App::new();

    app.add_plugins(SavePlugins)
        .register_type::<Transform>()
        .register_type::<Visibility>()
        .register_type::<CheckpointList>()
        .register_type::<Checkpoints>();

    app.world_mut().spawn(Transform::from_xyz(0.0, 1.0, 2.0));

    let snap = Snapshot::from_world(app.world());

    app.world_mut()
        .resource_mut::<Checkpoints>()
        .checkpoint(snap);

    let cps = app.world().resource::<Checkpoints>();
    let snap = cps.active().expect("No checkpoint found");
    let registry = app.world().resource::<AppTypeRegistry>().read();

    let ser = TypedReflectSerializer::new(snap, &registry);
    let out = json_serialize(&ser).expect("Failed to serialize");

    println!("{}", out);
    assert_eq!(out, CHECKPOINT_JSON);

    #[derive(Reflect, Resource, Default)]
    #[reflect(Resource, Default)]
    struct CheckpointList {
        items: Vec<Snapshot>,
    }

    let data = CheckpointList {
        items: vec![snap.clone()],
    };

    let ser = TypedReflectSerializer::new(&data, &registry);
    let out = json_serialize(&ser).expect("Failed to serialize");

    println!("{}", out);
    assert_eq!(out, CHECKPOINT_LIST_JSON);

    let mut snap = snap.clone();

    snap.resources
        .push(clone_reflect_value(&data, &registry).into());

    let ser = TypedReflectSerializer::new(&snap, &registry);
    let out = json_serialize(&ser).expect("Failed to serialize");

    println!("{}", out);
    assert_eq!(out, CHECKPOINT_LIST_NESTED_JSON);

    drop(registry);

    app.insert_resource(data);
    app.world_mut().clear_entities();

    let registry = app.world().resource::<AppTypeRegistry>().read();

    let snap = Snapshot::from_world(app.world());
    let ser = TypedReflectSerializer::new(&snap, &registry);
    let out = json_serialize(&ser).expect("Failed to serialize");

    println!("{}", out);
    assert_eq!(out, CHECKPOINT_SNAPSHOT_JSON);

    let snap = clone_reflect_value(&snap, &registry);
    let ser = TypedReflectSerializer::new(&*snap, &registry);
    let out = json_serialize(&ser).expect("Failed to serialize");

    println!("{}", out);
    assert_eq!(out, CHECKPOINT_SNAPSHOT_JSON);
}
