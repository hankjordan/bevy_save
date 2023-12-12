#![feature(impl_trait_in_assoc_type)]

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_save::{
    prelude::*,
    typed::{
        extract::{
            ExtractComponent,
            ExtractDeserialize,
            ExtractMapEntities,
            ExtractResource,
            ExtractSerialize,
        },
        SaveRegistry,
    },
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Player;

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Head;

#[derive(Component, Clone, Serialize, Deserialize)]
pub enum EnumComponent {
    A { x: f32, y: f32, z: f32 },
    B(f32, f32, f32),
    C,
}

fn setup(mut commands: Commands) {
    commands.spawn(EnumComponent::A {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    });

    commands.spawn(EnumComponent::B(4.0, 5.0, 6.0));

    commands.spawn(EnumComponent::C);

    commands
        .spawn((SpatialBundle::default(), Player))
        .with_children(|p| {
            p.spawn(Head);
        });

    // to reproduce the error, hit the following keys:
    // P         - Print debug info about heads and check their parent exists
    // ENTER     - Save
    // R         - Reset, delete all player entities
    // BACKSPACE - Load
    // P         - Print head info <== head will have an invalid parent
    println!("Controls:");
    println!("P: to print debug info on `Head` entities and to validate their parent exists");
    println!("R: to recursively delete all `Player` entities");
    println!("ENTER: Save");
    println!("BACKSPACE: Load");
}

struct TypedPipeline;

impl Pipeline for TypedPipeline {
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = &'a str;

    type Components =
        impl ExtractComponent + ExtractSerialize + ExtractDeserialize + ExtractMapEntities;
    type Resources =
        impl ExtractResource + ExtractSerialize + ExtractDeserialize + ExtractMapEntities;

    fn key(&self) -> Self::Key<'_> {
        "examples/saves/typed"
    }

    fn registry() -> SaveRegistry<Self::Components, Self::Resources> {
        SaveRegistry::new()
            //.component::<Head>()
            //.component::<Player>()
            //.component::<GlobalTransform>()
            //.component::<Transform>()
            .component::<EnumComponent>()
            //.reflect_component::<Parent>()
            //.reflect_component::<Children>()
            //.reflect_component::<InheritedVisibility>()
            //.reflect_component::<ViewVisibility>()
            //.reflect_component::<Visibility>()
    }

    fn capture(world: &bevy::prelude::World) -> Snapshot<Self::Components, Self::Resources> {
        Self::registry()
            .builder(world)
            .extract_entities_matching(|e| e.contains::<Player>() || e.contains::<Head>() || e.contains::<EnumComponent>())
            .build()
    }

    fn apply(world: &mut World, snapshot: &Snapshot<Self::Components, Self::Resources>) {
        snapshot.apply(world)
    }
}

fn handle_save_input(world: &mut World) {
    let keys = world.resource::<Input<KeyCode>>();

    if keys.just_released(KeyCode::Return) {
        info!("Save");
        world.save_typed(TypedPipeline).expect("Failed to save");
    } else if keys.just_released(KeyCode::Back) {
        info!("Load");
        todo!()
        //world.load(HeirarchyPipeline).expect("Failed to load");
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            file_path: "examples/assets".to_owned(),
            ..default()
        }))
        ////
        // Inspector
        .add_plugins(WorldInspectorPlugin::new())
        ////
        // Bevy Save
        .init_typed_pipeline::<TypedPipeline>()
        ////
        .add_systems(Startup, setup)
        .add_systems(Update, handle_save_input)
        ////
        .run();
}
