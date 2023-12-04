/*
use std::marker::PhantomData;

use bevy::{
    ecs::world::EntityRef,
    prelude::*,
};
use serde::{
    de::Visitor,
    ser::{
        SerializeMap,
        SerializeSeq,
        SerializeStruct,
    },
    Deserialize,
    Serialize,
};

use crate::Snapshot2;

#[test]
fn test_snapshot() {
    #[derive(Component, Clone, Serialize, Deserialize)]
    struct ExampleComponent {
        name: String,
    }

    #[derive(Component, Clone, Serialize, Deserialize)]
    struct OtherComponent;

    #[derive(Resource, Clone, Serialize, Deserialize)]
    struct SimpleResource {
        data: u32,
    }

    let mut app = App::new();
    let world = &mut app.world;

    world.spawn(ExampleComponent {
        name: "First".into(),
    });
    world.spawn((
        ExampleComponent {
            name: "Second".into(),
        },
        OtherComponent,
    ));
    world.spawn(OtherComponent);
    world.spawn(OtherComponent);

    world.insert_resource(SimpleResource { data: 42 });

    let registry = SaveRegistry::new()
        .register_component::<ExampleComponent>()
        .register_component::<OtherComponent>()
        .register_resource::<SimpleResource>();

    let snapshot = registry.builder(world).extract_all_entities().build();

    let mut buf = Vec::new();

    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);

    snapshot.serialize(&mut ser).unwrap();

    let output = std::str::from_utf8(&buf).unwrap();
    let expected = r#"{
    "entities": {
        "0": [
            {
                "name": "First"
            },
            null
        ],
        "1": [
            {
                "name": "Second"
            },
            {}
        ],
        "2": [
            null,
            {}
        ],
        "3": [
            null,
            {}
        ]
    },
    "resources": [
        {
            "data": 42
        }
    ]
}"#;

    assert_eq!(output, expected);

    let mut de = serde_json::Deserializer::from_str(output);

    let snapshot = registry.deserialize(&mut de).unwrap();

    let mut buf = Vec::new();

    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);

    snapshot.serialize(&mut ser).unwrap();

    let output = std::str::from_utf8(&buf).unwrap();

    assert_eq!(output, expected);
}
*/
