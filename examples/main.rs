//! A simple, console-only example of how to use `bevy_save`.

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_save::prelude::*;

#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct Balance {
    amount: usize,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Health {
    amount: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Player;

fn setup(mut commands: Commands) {
    commands.spawn((SpatialBundle::default(), Health { amount: 25.0 }, Player));
}

fn damage(buttons: Res<Input<MouseButton>>, mut players: Query<&mut Health, With<Player>>) {
    if buttons.just_released(MouseButton::Left) {
        for mut health in &mut players {
            health.amount -= 1.0;
        }
    }
}

fn heal(
    buttons: Res<Input<MouseButton>>,
    mut balance: ResMut<Balance>,
    mut players: Query<&mut Health, With<Player>>,
) {
    if buttons.just_released(MouseButton::Right) && balance.amount > 0 {
        for mut health in &mut players {
            health.amount += 1.0;
        }

        balance.amount -= 1;
    }
}

fn status(balance: Res<Balance>, players: Query<(Entity, &Health), Changed<Health>>) {
    if balance.is_changed() {
        info!("Balance: {:?} gp", balance.amount);
    }

    for (entity, health) in &players {
        info!("{:?}: {:?} hp", entity, health.amount);
    }
}

struct MainPipeline;

impl Pipeline for MainPipeline {
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "examples/saves/example"
    }
}

fn interact(world: &mut World) {
    let keys = world.resource::<Input<KeyCode>>();

    if keys.just_released(KeyCode::Space) {
        info!("Checkpoint");
        world.checkpoint::<MainPipeline>();
    } else if keys.just_released(KeyCode::Return) {
        info!("Save");
        world.save(MainPipeline).expect("Failed to save");
    } else if keys.just_released(KeyCode::Back) {
        info!("Load");
        world.load(MainPipeline).expect("Failed to load");
    } else if keys.just_released(KeyCode::Left) {
        info!("Rollback");
        world
            .rollback::<MainPipeline>(1)
            .expect("Failed to rollback");
    } else if keys.just_released(KeyCode::Right) {
        info!("Rollforward");
        world
            .rollback::<MainPipeline>(-1)
            .expect("Failed to rollforward");
    } else if keys.just_pressed(KeyCode::E) {
        info!("Info");

        for entity in world.iter_entities() {
            info!("Entity: {:?}", entity.id());

            for component_id in entity.archetype().components() {
                if let Some(component) = world.components().get_info(component_id) {
                    info!("{:?}: {:?}", entity.id(), component.name());
                }
            }
        }
    } else if keys.just_pressed(KeyCode::S) {
        info!("Spawn Entity");
        world.spawn_empty();
    }
}

fn main() {
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
        .insert_resource(Balance { amount: 42 })
        .register_saveable::<Balance>()
        .register_saveable::<Health>()
        .register_saveable::<Player>()
        // While it is still included in saves, the Balance resource will not rollback / rollforward alongside other types.
        // This could be used to track rollback state or to prevent players from making changes to their decisions during rollback.
        .ignore_rollback::<Balance>()
        .add_systems(Startup, setup)
        .add_systems(Update, (damage, heal, status, interact))
        .run();
}
