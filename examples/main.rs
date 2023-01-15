use bevy::prelude::*;
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

fn setup(
    mut commands: Commands,
) {
    commands
        .spawn((
            SpatialBundle::default(),
            Health { amount: 25.0 },
            Player,
        ));
}

fn damage(
    buttons: Res<Input<MouseButton>>,
    mut players: Query<&mut Health, With<Player>>,
) {
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

fn status(
    balance: Res<Balance>,
    players: Query<(Entity, &Health), Changed<Health>>,
) {
    if balance.is_changed() {
        info!("Balance: {:?} gp", balance.amount);
    }

    for (entity, health) in &players {
        info!("{:?}: {:?} hp", entity, health.amount);
    }
}

fn savestate(
    world: &mut World,
) {
    let keys = world.resource::<Input<KeyCode>>();

    if keys.just_released(KeyCode::Space) {
        info!("Checkpoint");
        world.checkpoint();
    } else if keys.just_released(KeyCode::Return) {
        info!("Save");
        world.save("example");
    } else if keys.just_released(KeyCode::Back) {
        info!("Load");
        world.load("example");
    } else if keys.just_released(KeyCode::Left) {
        info!("Rollback");
        world.rollback(1);
    } else if keys.just_released(KeyCode::Right) {
        info!("Rollforward");
        world.rollback(-1);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            asset_folder: "examples/assets".to_owned(),
            ..default()
        }))

        .insert_resource(Balance { amount: 42 })

        .register_saveable::<Balance>()
        .register_saveable::<Health>()
        .register_saveable::<Player>()

        // While it is still included in saves, the Balance resource will not rollback / rollforward alongside other types.
        // This could be used to track rollback state, or to prevent players from making changes to their decisions during rollback.
        .ignore_rollback::<Balance>()

        .add_startup_system(setup)

        .add_system(damage)
        .add_system(heal)
        .add_system(status)

        .add_system(savestate)

        .run();
}
