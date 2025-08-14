use bevy::{
    prelude::*,
    reflect::reflect_remote,
};
use bevy_save::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ComponentA {
    x: f32,
    y: f32,
    z: f32,
}

fn init_app() -> App {
    let mut app = App::new();

    app.register_type::<ComponentA>();

    let world = app.world_mut();

    world.spawn(ComponentA {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    });
    world.spawn(ComponentA {
        x: 4.0,
        y: 5.0,
        z: 6.0,
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
fn test_clone() {
    let app = init_app();
    let registry = app.world().resource::<AppTypeRegistry>().read();

    let a = Snapshot::from_world(app.world());
    let b = a.clone();

    let a_out = json_serialize(&a.serializer(&registry)).unwrap();
    let b_out = json_serialize(&b.serializer(&registry)).unwrap();

    println!("A, {:?}", a);
    println!("B, {:?}", b);

    assert_eq!(a_out, b_out);
}

struct Other {
    entity: Entity,
    values: Vec<f32>,
}

#[reflect_remote(Other)]
struct DynEntity {
    entity: Entity,
    values: Vec<f32>,
}
