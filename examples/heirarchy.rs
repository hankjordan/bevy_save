use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use bevy_save::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Head;

fn setup(mut commands: Commands) {
    commands
        .spawn((Player, Transform::default(), Visibility::default()))
        .with_children(|p| {
            p.spawn(Head);
        });

    commands.spawn(Camera2d);

    println!("Controls:");
    println!("P: to print debug info on `Head` entities and to validate their parent exists");
    println!("R: to recursively delete all `Player` entities");
    println!("ENTER: Save");
    println!("BACKSPACE: Load");
}

struct HeirarchyPipeline;

impl Pipeline for HeirarchyPipeline {
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "examples/saves/heirarchy"
    }

    fn capture(&self, builder: BuilderRef) -> Snapshot {
        builder
            .extract_entities_matching(|e| e.contains::<Player>() || e.contains::<Head>())
            .build()
    }

    fn apply(&self, world: &mut World, snapshot: &Snapshot) -> Result<(), bevy_save::Error> {
        snapshot
            .applier(world)
            .despawn::<Or<(With<Player>, With<Head>)>>()
            .apply()
    }
}

fn interact(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();

    if keys.just_released(KeyCode::Enter) {
        info!("Saving data");
        world.save(&HeirarchyPipeline).expect("Failed to save");
    } else if keys.just_released(KeyCode::Backspace) {
        info!("Loading data");
        world.load(&HeirarchyPipeline).expect("Failed to load");
    } else if keys.just_pressed(KeyCode::KeyE) {
        info!("Info");
        for entity in world.iter_entities() {
            info!("Entity: {:?}", entity.id());
            for component_id in entity.archetype().components() {
                if let Some(component) = world.components().get_info(component_id) {
                    info!("  {:?}: {:?}", entity.id(), component.name());
                }
            }
        }
    }
}

fn handle_keys(
    keys: Res<ButtonInput<KeyCode>>,
    head_query: Query<(Entity, &ChildOf)>,
    despawn_query: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    // Print head debug info, check that all heads have a valid parent
    if keys.just_released(KeyCode::KeyP) {
        println!("{} Heads", head_query.iter().len());
        for (entity, child_of) in &head_query {
            println!("  Head {:?} has parent: {:?}", entity, child_of.parent());
            if commands.get_entity(child_of.parent()).is_err() {
                println!("    X - Head parent does not exist!");
            } else {
                println!("    Ok - Head parent exists, all good")
            }
        }
    }

    // Reset, delete all entities
    if keys.just_released(KeyCode::KeyR) {
        for entity in &despawn_query {
            commands.entity(entity).despawn();
        }
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
        .register_type::<Player>()
        .register_type::<Head>()
        // Systems
        .add_systems(Startup, setup)
        .add_systems(Update, (interact, handle_keys))
        .run();
}
