use bevy::{
    prelude::*,
    scene::{
        serde::{
            SceneDeserializer,
            SceneSerializer,
        },
        DynamicEntity,
    },
};
use serde::{
    de::DeserializeSeed,
    Serialize,
};

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Parent {
    value: Option<Child>,
}

#[derive(Reflect)]
struct Child {
    value: u32,
}

#[test]
fn test_upstream() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    app.register_type::<Parent>()
        .register_type::<Child>()
        .register_type::<Option<Child>>();

    let world = &mut app.world;

    let registry = world.resource::<AppTypeRegistry>();

    let scene = DynamicScene {
        resources: vec![],
        entities: vec![DynamicEntity {
            entity: Entity::from_raw(0),
            components: vec![Parent { value: None }.clone_value()],
        }],
    };

    let ser = SceneSerializer::new(&scene, registry);

    let mut buf = Vec::new();

    ser.serialize(&mut serde_json::Serializer::new(&mut buf))
        .expect("Failed to serialize");

    let de = SceneDeserializer {
        type_registry: &registry.read(),
    };

    de.deserialize(&mut serde_json::Deserializer::from_slice(&buf))
        .expect("Failed to deserialize");
}
