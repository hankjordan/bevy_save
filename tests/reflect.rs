use bevy::prelude::*;
use bevy_save::{
    prelude::*,
    typed::SaveRegistry,
};
use serde::{
    Deserialize,
    Serialize,
};

// Typed<T> and Dynamic<T> should serialize identically

#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub enum EnumComponent {
    A {
        x: f32,
        y: f32,
        z: f32,
    },
    B(f32, f32, f32),
    #[default]
    C,
}

#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct StructComponent {
    name: String,
    list: Vec<f32>,
    unit: (),
}

#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct TupleComponent(u64);

#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct UnitComponent;

fn app() -> App {
    let mut app = App::new();

    app ////
        .add_plugins((MinimalPlugins, SavePlugins))
        .register_type::<EnumComponent>()
        .register_type::<StructComponent>()
        .register_type::<Vec<f32>>()
        .register_type::<UnitComponent>();

    let world = &mut app.world;

    world.spawn((
        EnumComponent::A {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        UnitComponent,
    ));
    world.spawn((EnumComponent::B(4.0, 5.0, 6.0), StructComponent {
        name: "Hello, world!".into(),
        list: vec![1.0, 2.0, 3.0],
        unit: (),
    }));
    world.spawn((EnumComponent::C, TupleComponent(5)));

    app
}

#[test]
fn test_json() {
    let app = app();
    let world = &app.world;

    let typed = SaveRegistry::new()
        .component::<EnumComponent>()
        .component::<StructComponent>()
        .component::<TupleComponent>()
        .component::<UnitComponent>()
        .builder(world)
        .extract_all_entities()
        .build();

    let mut buf = Vec::new();
    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);

    typed.serialize(&mut ser).unwrap();

    let typed = String::from_utf8(buf).unwrap();
    let expected = r#"{
    "entities": {
        "0": [
            {
                "A": {
                    "x": 1.0,
                    "y": 2.0,
                    "z": 3.0
                }
            },
            null,
            null,
            {}
        ],
        "1": [
            {
                "B": [
                    4.0,
                    5.0,
                    6.0
                ]
            },
            {
                "name": "Hello, world!",
                "list": [
                    1.0,
                    2.0,
                    3.0
                ],
                "unit": null
            },
            null,
            null
        ],
        "2": [
            "C",
            null,
            5,
            null
        ]
    },
    "resources": []
}"#;

    assert_eq!(typed, expected);

    let reflect = SaveRegistry::new()
        .reflect_component::<EnumComponent>()
        .reflect_component::<StructComponent>()
        .reflect_component::<TupleComponent>()
        .reflect_component::<UnitComponent>()
        .builder(world)
        .extract_all_entities()
        .build();

    let mut buf = Vec::new();
    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);

    reflect.serialize(&mut ser).unwrap();

    let reflect = String::from_utf8(buf).unwrap();

    assert_eq!(typed, reflect);
}

#[test]
fn test_mp() {
    let app = app();
    let world = &app.world;

    let typed = SaveRegistry::new()
        .component::<EnumComponent>()
        .component::<StructComponent>()
        .component::<TupleComponent>()
        .component::<UnitComponent>()
        .builder(world)
        .extract_all_entities()
        .build();

    let mut buf = Vec::new();
    let mut ser = rmp_serde::Serializer::new(&mut buf);

    typed.serialize(&mut ser).unwrap();

    let typed = buf;
    let expected = [
        146, 131, 0, 148, 129, 161, 65, 147, 202, 63, 128, 0, 0, 202, 64, 0, 0, 0, 202, 64, 64, 0,
        0, 192, 192, 128, 1, 148, 129, 161, 66, 147, 202, 64, 128, 0, 0, 202, 64, 160, 0, 0, 202,
        64, 192, 0, 0, 147, 173, 72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33, 147,
        202, 63, 128, 0, 0, 202, 64, 0, 0, 0, 202, 64, 64, 0, 0, 192, 192, 192, 2, 148, 161, 67,
        192, 5, 192, 144,
    ];

    assert_eq!(typed, expected);

    let reflect = SaveRegistry::new()
        .reflect_component::<EnumComponent>()
        .reflect_component::<StructComponent>()
        .reflect_component::<TupleComponent>()
        .reflect_component::<UnitComponent>()
        .builder(world)
        .extract_all_entities()
        .build();

    let mut buf = Vec::new();
    let mut ser = rmp_serde::Serializer::new(&mut buf);

    reflect.serialize(&mut ser).unwrap();

    let reflect = buf;

    assert_eq!(typed, reflect);
}
