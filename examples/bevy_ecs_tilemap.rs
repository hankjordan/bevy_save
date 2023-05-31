//! A game of life simulator.
//! Modified to demonstrate integration of `bevy_save`.

use bevy::prelude::*;
use bevy_ecs_tilemap::{
    helpers::square_grid::neighbors::Neighbors,
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_save::prelude::*;

fn setup(world: &mut World) {
    if let Err(e) = world.load("gol") {
        info!("Failed to load: {:?}", e);

        let mut system = IntoSystem::into_system(startup);

        system.initialize(world);
        system.run((), world);
        system.apply_buffers(world);
    }

    let mut system = IntoSystem::into_system(finish_setup);

    system.initialize(world);
    system.run((), world);
    system.apply_buffers(world);
}

fn finish_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tile_storage_query: Query<Entity, With<TileStorage>>,
) {
    commands.spawn(Camera2dBundle::default());
    
    let tilemap_entity = tile_storage_query.single();
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    commands
        .entity(tilemap_entity)
        .insert((TilemapTexture::Single(texture_handle), LastUpdate(0.0)));
}

fn startup(mut commands: Commands) {
    let map_size = TilemapSize { x: 32, y: 32 };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    let mut i = 0;
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    visible: TileVisible(i % 2 == 0 || i % 7 == 0),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
            i += 1;
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    commands.entity(tilemap_entity).insert((TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    },));
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct LastUpdate(f64);

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut tile_storage_query: Query<(&TileStorage, &TilemapSize, &mut LastUpdate)>,
    tile_query: Query<(Entity, &TilePos, &TileVisible)>,
) {
    let current_time = time.elapsed_seconds_f64();
    let (tile_storage, map_size, mut last_update) = tile_storage_query.single_mut();
    if current_time - last_update.0 > 0.1 {
        for (entity, position, visibility) in tile_query.iter() {
            let neighbor_count =
                Neighbors::get_square_neighboring_positions(position, map_size, true)
                    .entities(tile_storage)
                    .iter()
                    .filter(|neighbor| {
                        let tile_component =
                            tile_query.get_component::<TileVisible>(**neighbor).unwrap();
                        tile_component.0
                    })
                    .count();

            let was_alive = visibility.0;

            let is_alive = match (was_alive, neighbor_count) {
                (true, x) if x < 2 => false,
                (true, 2) | (true, 3) => true,
                (true, x) if x > 3 => false,
                (false, 3) => true,
                (otherwise, _) => otherwise,
            };

            if is_alive && !was_alive {
                commands.entity(entity).insert(TileVisible(true));
            } else if !is_alive && was_alive {
                commands.entity(entity).insert(TileVisible(false));
            }
        }
        last_update.0 = current_time;
    }
}

fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    for (mut transform, mut ortho) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::A) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::S) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::Z) {
            ortho.scale += 0.1;
        }

        if keyboard_input.pressed(KeyCode::X) {
            ortho.scale -= 0.1;
        }

        if ortho.scale < 0.5 {
            ortho.scale = 0.5;
        }

        let z = transform.translation.z;
        transform.translation += time.delta_seconds() * direction * 500.;
        // Important! We need to restore the Z values when moving the camera around.
        // Bevy has a specific camera setup and this can mess with how our layers are shown.
        transform.translation.z = z;
    }
}

fn save(world: &World) {
    let keys = world.resource::<Input<KeyCode>>();

    if keys.just_released(KeyCode::Return) {
        world.save("gol").expect("Failed to save");
    }
}

#[rustfmt::skip]
fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Game of Life Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    asset_folder: "examples/assets".to_owned(),
                    ..default()
                }),
        )
        

        // Inspector
        .add_plugin(WorldInspectorPlugin::new())

        // Bevy Save
        .add_plugins(SavePlugins)

        .add_plugin(TilemapPlugin)

        // Setup
        .add_startup_system(setup)

        // Systems
        .add_system(movement)
        .add_system(update)
        .add_system(save)

        .run();
}
