use bevy::{
    ecs::{
        entity::{
            EntityHashMap,
            MapEntities,
        },
        reflect::ReflectMapEntities,
    },
    prelude::*,
    reflect::{
        GetTypeRegistration,
        serde::{
            TypedReflectDeserializer,
            TypedReflectSerializer,
        },
    },
};
use bevy_save::{
    prelude::*,
    reflect::{
        DynamicValue,
        ReflectMap,
    },
};
use serde::{
    Deserialize,
    Serialize,
    de::DeserializeSeed,
};

#[derive(Resource, Reflect, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[reflect(opaque)]
#[reflect(Resource, MapEntities, Serialize, Deserialize)]
struct Example {
    values: Vec<(u32, Entity)>,
}

impl MapEntities for Example {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.values.iter_mut().for_each(|(_, e)| {
            *e = entity_mapper.get_mapped(*e);
        });
    }
}

fn init_app() -> App {
    let mut app = App::new();

    app.add_plugins(SavePlugins).register_type::<Example>();

    app
}

fn json_serialize<T: Serialize>(value: &T) -> String {
    let mut buf = Vec::new();
    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);
    value.serialize(&mut ser).expect("Failed to serialize");
    String::from_utf8(buf).expect("Invalid string")
}

#[test]
fn test_opaque_clone() {
    let orig = Example {
        values: [
            (1, Entity::from_raw(10)),
            (2, Entity::from_raw(20)),
            (3, Entity::from_raw(30)),
        ]
        .into(),
    };
    let reflect = Box::new(orig).into_partial_reflect();

    assert!(
        reflect.reflect_clone().is_err(),
        "opaque should not implement `reflect_clone`"
    );
}

#[test]
fn test_opaque_value() {
    let app = init_app();
    let registry = app.world().resource::<AppTypeRegistry>().read();

    let orig = Example {
        values: [
            (1, Entity::from_raw(10)),
            (2, Entity::from_raw(20)),
            (3, Entity::from_raw(30)),
        ]
        .into(),
    };

    let reflect = DynamicValue::from_reflect(&orig).expect("FromReflect failed");

    assert_eq!(Example::from_reflect(&reflect).unwrap(), orig);

    let json_a = json_serialize(&TypedReflectSerializer::new(&orig, &registry));
    let json_b = json_serialize(&TypedReflectSerializer::new(&reflect, &registry));

    let reg = Example::get_type_registration();
    let seed = TypedReflectDeserializer::new(&reg, &registry);
    let mut de = serde_json::Deserializer::from_str(&json_a);
    let out_a = seed.deserialize(&mut de).expect("Failed to deserialize");

    assert_eq!(Example::take_from_reflect(out_a).unwrap(), orig);

    // `DynamicValue` cannot be deserialized as `DynamicValue`
    let reg = Example::get_type_registration();
    let seed = TypedReflectDeserializer::new(&reg, &registry);
    let mut de = serde_json::Deserializer::from_str(&json_b);
    let out_b = seed.deserialize(&mut de).expect("Failed to deserialize");

    assert_eq!(Example::take_from_reflect(out_b).unwrap(), orig);
}

#[test]
fn test_opaque_map() {
    let app = init_app();
    let registry = app.world().resource::<AppTypeRegistry>().read();

    let map_a: ReflectMap = vec![
        Box::new(Example {
            values: [
                (1, Entity::from_raw(10)),
                (2, Entity::from_raw(20)),
                (3, Entity::from_raw(30)),
            ]
            .into(),
        })
        .into_partial_reflect(),
    ]
    .into();

    let json_a = json_serialize(&TypedReflectSerializer::new(&map_a, &registry));

    let reg = ReflectMap::get_type_registration();
    let seed = TypedReflectDeserializer::new(&reg, &registry);
    let mut de = serde_json::Deserializer::from_str(&json_a);
    let out = seed.deserialize(&mut de).expect("Failed to deserialize");

    let map_b = ReflectMap::take_from_reflect(out).unwrap();

    let json_b = json_serialize(&TypedReflectSerializer::new(&map_b, &registry));

    assert_eq!(json_a, json_b);
}

#[test]
fn test_opaque_snapshot() {
    let mut app = init_app();

    app.insert_resource(Example {
        values: [
            (1, Entity::from_raw(10)),
            (2, Entity::from_raw(20)),
            (3, Entity::from_raw(30)),
        ]
        .into(),
    });

    let registry = app.world().resource::<AppTypeRegistry>().read();

    let snap_a = Snapshot::builder(app.world())
        .extract_resource::<Example>()
        .build();

    let json_a = json_serialize(&snap_a.serializer(&registry));

    let seed = Snapshot::deserializer(&registry);
    let mut de = serde_json::Deserializer::from_str(&json_a);
    let snap_b = seed.deserialize(&mut de).expect("Failed to deserialize");

    let json_b = json_serialize(&snap_b.serializer(&registry));

    assert_eq!(json_a, json_b);

    drop(registry);

    let mut map: EntityHashMap<Entity> = [
        (Entity::from_raw(10), Entity::from_raw(100)),
        (Entity::from_raw(20), Entity::from_raw(200)),
        (Entity::from_raw(30), Entity::from_raw(300)),
    ]
    .into_iter()
    .collect();

    snap_b
        .applier(app.world_mut())
        .entity_map(&mut map)
        .apply()
        .expect("Failed to apply");

    let snap_c = Snapshot::builder(app.world())
        .extract_resource::<Example>()
        .build();

    assert_eq!(
        Example::from_reflect(&**snap_c.resources().first().unwrap()).unwrap(),
        Example {
            values: [
                (1, Entity::from_raw(100)),
                (2, Entity::from_raw(200)),
                (3, Entity::from_raw(300))
            ]
            .into()
        }
    );
}
