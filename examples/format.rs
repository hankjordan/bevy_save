use std::io::{
    Read,
    Write,
};

use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use bevy_save::{
    Error,
    prelude::*,
};
use io_adapters::WriteExtension;
use serde::{
    Serialize,
    de::DeserializeSeed,
};

pub struct RONFormat;

impl Format for RONFormat {
    fn extension() -> &'static str {
        ".ron"
    }

    fn serialize<W: Write, T: Serialize>(writer: W, value: &T) -> Result<(), Error> {
        let mut ser = ron::Serializer::new(
            writer.write_adapter(),
            Some(ron::ser::PrettyConfig::default()),
        )
        .map_err(Error::saving)?;

        value.serialize(&mut ser).map_err(Error::saving)
    }

    fn deserialize<R: Read, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
        reader: R,
        seed: S,
    ) -> Result<T, Error> {
        ron::options::Options::default()
            .from_reader_seed(reader, seed)
            .map_err(Error::loading)
    }
}

pub struct RONPipeline;

impl Pipeline for RONPipeline {
    type Backend = DefaultDebugBackend;
    type Format = RONFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "examples/saves/format"
    }

    fn capture(&self, builder: BuilderRef) -> Snapshot {
        builder
            .allow::<Transform>()
            .allow::<ExampleComponent>()
            // This is just one way to prevent extracting the camera.
            // Instead of using this match, you could use a marker component
            // and only extract entities with that marker component.
            .extract_entities_matching(|e| !e.contains::<Camera>())
            .clear_empty()
            .build()
    }

    fn apply(&self, world: &mut World, snapshot: &Snapshot) -> Result<(), Error> {
        snapshot
            .applier(world)
            .despawn::<(With<Transform>, Without<Camera>)>()
            .apply()
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ExampleComponent {
    float: f32,
    string: String,
}

fn setup(mut commands: Commands) {
    commands.spawn_empty();
    commands.spawn((Transform::from_xyz(1.0, 2.0, 3.0),));
    commands.spawn((Transform::from_xyz(4.0, 5.0, 6.0), ExampleComponent {
        float: 64.0,
        string: "Hello, world!".into(),
    }));
    commands.spawn(Camera2d);
}

fn handle_save_input(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();

    if keys.just_released(KeyCode::Enter) {
        info!("Saving data");
        world.save(&RONPipeline).expect("Failed to save");
    } else if keys.just_released(KeyCode::Backspace) {
        info!("Loading data");
        world.load(&RONPipeline).expect("Failed to load");
    }
}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            file_path: "examples/assets".to_owned(),
            ..default()
        }))
        // Inspector
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        // Bevy Save
        .add_plugins(SavePlugins)
        // Register types
        .register_type::<ExampleComponent>()
        // Systems
        .add_systems(Startup, setup)
        .add_systems(Update, handle_save_input)
        .run();
}
