use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_save::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Head;

fn setup(mut commands: Commands) {
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
    // P         - Print head info
    println!("Controls:");
    println!("P: to print debug info on `Head` entities and to validate their parent exists");
    println!("R: to recursively delete all `Player` entities");
    println!("ENTER: Save");
    println!("BACKSPACE: Load");
}

struct HeirarchyPipeline;

impl DynamicPipeline for HeirarchyPipeline {
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "examples/saves/heirarchy"
    }

    fn capture(builder: DynamicSnapshotBuilder) -> DynamicSnapshot {
        builder
            .extract_entities_matching(|e| e.contains::<Player>() || e.contains::<Head>())
            .build()
    }

    fn apply(world: &mut World, snapshot: &DynamicSnapshot) -> Result<(), bevy_save::Error> {
        snapshot
            .applier(world)
            .despawn::<Or<(With<Player>, With<Head>)>>()
            .apply()
    }
}

fn interact(world: &mut World) {
    let keys = world.resource::<Input<KeyCode>>();

    if keys.just_released(KeyCode::Return) {
        info!("Save");
        world.save(HeirarchyPipeline).expect("Failed to save");
    } else if keys.just_released(KeyCode::Back) {
        info!("Load");
        world.load(HeirarchyPipeline).expect("Failed to load");
    } else if keys.just_pressed(KeyCode::E) {
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
    keys: Res<Input<KeyCode>>,
    head_query: Query<(Entity, &Parent)>,
    despawn_query: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    // Print head debug info, check that all heads have a valid parent
    if keys.just_released(KeyCode::P) {
        println!("{} Heads", head_query.iter().len());
        for (entity, parent) in &head_query {
            println!("  Head {:?} has parent: {:?}", entity, parent.get());
            if commands.get_entity(parent.get()).is_none() {
                println!("    X - Head parent does not exist!");
            } else {
                println!("    Ok - Head parent exists, all good")
            }
        }
    }

    // Reset, delete all entities
    if keys.just_released(KeyCode::R) {
        for entity in &despawn_query {
            commands.entity(entity).despawn_recursive();
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
        .add_plugins(WorldInspectorPlugin::new())
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
