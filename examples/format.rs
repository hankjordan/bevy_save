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
    prelude::*,
    Error,
};
use io_adapters::WriteExtension;
use serde::{
    de::DeserializeSeed,
    Serialize,
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

    fn capture(&self, builder: SnapshotBuilder) -> Snapshot {
        builder
            .allow::<Transform>()
            .allow::<ExampleComponent>()
            .extract_all_entities()
            .clear_empty()
            .build()
    }

    fn apply(&self, world: &mut World, snapshot: &Snapshot) -> Result<(), Error> {
        snapshot.apply(world)
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
}

fn extract(mut commands: Commands) {
    commands.save(RONPipeline);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            file_path: "examples/assets".to_owned(),
            ..default()
        }))
        // Inspector
        .add_plugins((
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            WorldInspectorPlugin::new(),
        ))
        // Bevy Save
        .add_plugins(SavePlugins)
        // Register types
        .register_type::<ExampleComponent>()
        // Systems
        .add_systems(Startup, (setup, extract))
        .run();
}
