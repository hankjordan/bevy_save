//! An example of how to implement your own `Pipeline`.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_save::prelude::*;

#[derive(Clone, Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct TileMap {
    map: HashMap<TilePosition, Entity>,
}

#[derive(Clone, Copy, Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub enum Tile {
    #[default]
    Empty,
    Grass,
    Dirt,
    Stone,
    IronOre,
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Reflect)]
#[reflect(Hash)]
pub struct TilePosition {
    x: i32,
    y: i32,
}

fn setup_world(mut commands: Commands) {
    let map = TileMap {
        map: ["xg", "ds"]
            .into_iter()
            .rev()
            .enumerate()
            .flat_map(|(y, r)| {
                r.chars()
                    .enumerate()
                    .map(move |(x, t)| (x as i32, y as i32, t))
            })
            .map(|(x, y, t)| (TilePosition { x, y }, t))
            .map(|(p, t)| {
                (p, match t {
                    'x' => Tile::Empty,
                    'g' => Tile::Grass,
                    'd' => Tile::Dirt,
                    's' => Tile::Stone,
                    'i' => Tile::IronOre,
                    _ => panic!(),
                })
            })
            .map(|(p, t)| (p, commands.spawn(t).id()))
            .collect(),
    };

    commands.insert_resource(map);
}

fn display_world(keys: Res<Input<KeyCode>>, map: Res<TileMap>, tiles: Query<&Tile>) {
    if keys.just_released(KeyCode::Space) {
        println!("Count: {:?}", tiles.iter().len());

        for (position, entity) in &map.map {
            let tile = tiles.get(*entity).expect("Invalid entity");
            println!("{entity:?}: {position:?} {tile:?}");
        }
    }
}

fn handle_save_input(world: &mut World) {
    let keys = world.resource::<Input<KeyCode>>();

    // Using DebugPipeline as the argument for save/load, we can save locally with JSON.

    if keys.just_released(KeyCode::Return) {
        // Save every tile individually.
        for position in world.resource::<TileMap>().map.keys() {
            world.save(TilePipeline(*position)).expect("Failed to save");
        }
    } else if keys.just_released(KeyCode::Back) {
        // For ease of implementation, let's just load the origin.
        world
            .load(TilePipeline(TilePosition { x: 0, y: 0 }))
            .expect("Failed to load");
    }
}

pub struct TilePipeline(pub TilePosition);

impl Pipeline for TilePipeline {
    type Backend = DefaultDebugBackend;
    type Saver = DefaultDebugSaver;
    type Loader = DefaultDebugLoader;

    type Key = String;

    fn capture(&self, world: &World) -> Snapshot {
        Snapshot::builder(world)
            .extract_entity(
                *world
                    .resource::<TileMap>()
                    .map
                    .get(&self.0)
                    .expect("Could not find tile"),
            )
            .build()
    }

    fn apply(world: &mut World, snapshot: Snapshot) -> Result<(), bevy_save::Error> {
        snapshot
            .applier(world)
            .despawn(DespawnMode::None)
            .mapping(MappingMode::Strict)
            .apply()
    }

    fn key(self) -> Self::Key {
        format!("examples/saves/pipeline/{}-{}.json", self.0.x, self.0.y)
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
        // Register our types as saveable
        .register_saveable::<TileMap>()
        .register_saveable::<TilePosition>()
        .register_saveable::<Tile>()
        // Bevy's reflection requires we register each generic instance of a type individually
        // Note that we only need to register it in the AppTypeRegistry and not in the SaveableRegistry
        .register_type::<HashMap<TilePosition, Entity>>()
        // Resources
        .init_resource::<TileMap>()
        // Systems
        .add_systems(Startup, setup_world)
        .add_systems(Update, (display_world, handle_save_input))
        .run();
}
