//! An example of how to use `bevy_save` to save/load world state in other formats.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_save::prelude::*;

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct FancyMap {
    map: HashMap<String, i32>,
    float: f32,
    bool: bool,
}

#[derive(Component, Deref, DerefMut, Reflect, Default)]
#[reflect(Component)]
pub struct Velocity(Vec2);

fn setup_world(mut commands: Commands) {
    commands.spawn((
        SpatialBundle::from_transform(Transform::from_xyz(0.0, 1.0, 2.0)),
        Velocity(Vec2::new(1.0, 2.0)),
    ));

    commands.spawn((
        SpatialBundle::from_transform(Transform::from_xyz(-4.0, 1.0, 2.0)),
        Velocity(Vec2::new(16.0, 2.0)),
    ));

    commands.spawn((
        SpatialBundle::from_transform(Transform::from_xyz(0.0, 4.0, 2.0)),
        Velocity(Vec2::new(1.0, -2.0)),
    ));

    commands.spawn((
        SpatialBundle::from_transform(Transform::from_xyz(0.0, 1.0, 8.0)),
        Velocity(Vec2::new(0.0, -2.0)),
    ));
}

fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn handle_save_input(world: &mut World) {
    let keys = world.resource::<Input<KeyCode>>();

    // Now that we've registered our custom saver and loader,
    // `bevy_save` will transparently save and load with our custom format.

    if keys.just_released(KeyCode::Return) {
        world.save("json").expect("Failed to save");
    } else if keys.just_released(KeyCode::Back) {
        world.load("json").expect("Failed to load");
    }
}

struct JSONSaver;

impl Saver for JSONSaver {
    fn serializer<'w>(&self, writer: Writer<'w>) -> IntoSerializer<'w> {
        IntoSerializer::erase(serde_json::Serializer::pretty(writer))
    }
}

struct JSONLoader;

impl Loader for JSONLoader {
    fn deserializer<'r, 'de>(&self, reader: Reader<'r>) -> IntoDeserializer<'r, 'de> {
        IntoDeserializer::erase(serde_json::Deserializer::from_reader(reader))
    }
}

fn main() {
    let mut fancy_map = FancyMap::default();

    fancy_map.map.insert("MyKey".into(), 123);
    fancy_map.map.insert("Another".into(), 456);
    fancy_map.map.insert("More!".into(), -555);
    fancy_map.float = 42.005;
    fancy_map.bool = true;

    App::new()
        .add_plugins((
            DefaultPlugins.build().set(AssetPlugin {
                file_path: "examples/assets".to_owned(),
                ..default()
            }),
            // Inspector
            WorldInspectorPlugin::new(),
            // Bevy Save
            SavePlugins,
        ))

        // Override the AppSaver and AppLoader with our custom saver and loader
        .insert_resource(AppSaver::new(JSONSaver))
        .insert_resource(AppLoader::new(JSONLoader))

        // Register our types as saveable
        .register_saveable::<FancyMap>()
        .register_saveable::<Velocity>()

        // Bevy's reflection requires we register each generic instance of a type individually
        // Note that we only need to register it in the AppTypeRegistry and not in the SaveableRegistry
        .register_type::<HashMap<String, i32>>()

        // Resources
        .insert_resource(fancy_map)

        // Systems
        .add_systems(Startup, setup_world)
        .add_systems(Update, (apply_velocity, handle_save_input))
        
        .run();
}
