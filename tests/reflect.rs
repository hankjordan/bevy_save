use bevy::prelude::*;
use bevy_save::{prelude::*, typed::SaveRegistry};
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

#[test]
fn test_typed_reflect() {
    let mut app = App::new();

    app ////
        .add_plugins((MinimalPlugins, SavePlugins))
        .register_type::<EnumComponent>();

    let world = &mut app.world;

    world.spawn(EnumComponent::A {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    });
    world.spawn(EnumComponent::B(4.0, 5.0, 6.0));
    world.spawn(EnumComponent::C);

    let typed = SaveRegistry::new()
        .component::<EnumComponent>()
        .builder(world)
        .extract_all_entities()
        .build();

    let mut buf = Vec::new();
    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);

    typed.serialize(&mut ser).unwrap();

    let typed = String::from_utf8(buf).unwrap();

    let reflect = SaveRegistry::new()
        .reflect_component::<EnumComponent>()
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
