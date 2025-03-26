#![allow(clippy::expect_fun_call)]

use bevy::{
    diagnostic::DiagnosticsPlugin,
    prelude::*,
    reflect::{
        serde::{
            ReflectDeserializer,
            ReflectSerializer,
            TypedReflectDeserializer,
            TypedReflectSerializer,
        },
        TypeRegistry,
    },
    render::{
        settings::{
            RenderCreation,
            WgpuSettings,
        },
        RenderPlugin,
    },
    winit::WinitPlugin,
};
use bevy_save::{
    prelude::*,
    serde::{
        SnapshotDeserializer,
        SnapshotSerializer,
    },
};
use serde::{
    de::DeserializeSeed,
    Deserialize,
    Serialize,
};

fn init_app() -> (App, Vec<u64>) {
    let mut app = App::new();

    app //
        .add_plugins((MinimalPlugins, SavePlugins))
        .register_type::<Transform>()
        .register_type::<Visibility>();

    let world = app.world_mut();

    let ids = vec![
        world.spawn(()).id().to_bits(),
        world
            .spawn(Transform::from_xyz(1.0, 2.0, 3.0))
            .id()
            .to_bits(),
    ];

    (app, ids)
}

fn extract(world: &World) -> Snapshot {
    Snapshot::builder(world).extract_all_entities().build()
}

#[derive(Serialize, Deserialize)]
struct ExampleTransform {
    position: Vec3,
}

#[test]
fn test_transforms() {
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

    let deserializer = SnapshotDeserializer {
        registry: &registry,
    };

    let mut de = serde_json::Deserializer::from_str(&output);

    let _ = deserializer.deserialize(&mut de).unwrap();
}

trait SerDe {
    type Error: std::fmt::Debug;

    fn ser<T>(value: &T) -> Result<String, Self::Error>
    where
        T: Serialize;

    fn de<D, T>(seed: D, text: &str) -> Result<T, Self::Error>
    where
        D: for<'de> DeserializeSeed<'de, Value = T>;
}

struct Json;

impl SerDe for Json {
    type Error = serde_json::Error;

    fn ser<T>(value: &T) -> Result<String, serde_json::Error>
    where
        T: Serialize,
    {
        serde_json::to_string_pretty(value)
    }

    fn de<'a, D, T>(de: D, text: &'a str) -> Result<T, serde_json::Error>
    where
        D: for<'de> DeserializeSeed<'de, Value = T>,
    {
        let mut deserializer = serde_json::Deserializer::from_str(text);
        de.deserialize(&mut deserializer)
    }
}

fn roundtrip_registered<S>(registry: &TypeRegistry, erased: bool)
where
    S: SerDe,
{
    for ty in registry.iter() {
        if !ty.contains::<ReflectSerialize>() || !ty.contains::<ReflectDeserialize>() {
            continue;
        }

        let type_path = ty.type_info().type_path();

        let Some(reflect_default) = ty.data::<ReflectDefault>() else {
            //println!("No default for {:?}", type_path);
            continue;
        };

        let default = reflect_default.default();
        let reflect = default.as_partial_reflect();

        let data = if erased {
            let value = ReflectSerializer::new(reflect, registry);
            S::ser(&value)
        } else {
            let value = TypedReflectSerializer::new(reflect, registry);
            S::ser(&value)
        }
        .expect(&format!("Failed to serialize {:?}", type_path));

        let output = if erased {
            let de = ReflectDeserializer::new(registry);
            S::de(de, &data)
        } else {
            let seed = TypedReflectDeserializer::new(ty, registry);
            S::de(seed, &data)
        }
        .expect(&format!(
            "Failed to deserialize {:?}\n{}\n",
            type_path, data
        ));

        //println!("{:?}: {data}", type_path);
        assert!(default.reflect_partial_eq(&*output).unwrap_or(true));
    }
}

fn build_registry_app() -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<WinitPlugin>()
            .disable::<DiagnosticsPlugin>()
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    backends: None,
                    ..default()
                }),
                ..default()
            }),
    );

    app
}

#[test]
fn test_builtin_types() {
    let app = build_registry_app();

    let registry = app.world().resource::<AppTypeRegistry>().read();

    roundtrip_registered::<Json>(&registry, true);
    roundtrip_registered::<Json>(&registry, false);
}

const TRANSFORM_JSON: &str = r#"
{
  "bevy_transform::components::transform::Transform": {
    "translation": [
      1.0,
      2.0,
      3.0
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
}"#;

const TRANSFORM_TYPED_JSON: &str = r#"
{
  "translation": [
    1.0,
    2.0,
    3.0
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
}"#;

const TRANSFORM_SNAPSHOT_JSON: &str = r#"
{
  "entities": {
    "4294967296": {
      "components": {
        "bevy_transform::components::transform::Transform": {
          "translation": [
            1.0,
            2.0,
            3.0
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

#[test]
fn transform_json() {
    let value = Transform::from_xyz(1.0, 2.0, 3.0);

    let mut app = App::new();

    app.register_type::<Transform>();

    app.world_mut().spawn(value);

    let registry = app.world().resource::<AppTypeRegistry>().read();

    let ser = ReflectSerializer::new(&value, &registry);
    let data_erased = serde_json::to_string_pretty(&ser).unwrap();

    assert_eq!(TRANSFORM_JSON, format!("\n{data_erased}"));

    let ser = TypedReflectSerializer::new(&value, &registry);
    let data_typed = serde_json::to_string_pretty(&ser).unwrap();

    assert_eq!(TRANSFORM_TYPED_JSON, format!("\n{data_typed}"));

    let snapshot = Snapshot::builder(app.world())
        .extract_all_entities()
        .build();

    let ser = snapshot.serializer(&registry);

    let output = serde_json::to_string_pretty(&ser).unwrap();

    assert_eq!(TRANSFORM_SNAPSHOT_JSON, format!("\n{output}"));
}
