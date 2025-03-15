//! An example of how to implement your own `Pipeline`.

use bevy::{
    ecs::entity::EntityHashMap,
    prelude::*,
    utils::HashMap,
};
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

#[derive(Clone, Copy, Component, Debug, Default, Hash, PartialEq, Eq, Reflect)]
#[reflect(Component, Hash)]
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

fn display_world(keys: Res<ButtonInput<KeyCode>>, map: Res<TileMap>, tiles: Query<&Tile>) {
    if keys.just_released(KeyCode::Space) {
        println!("Count: {:?}", tiles.iter().len());

        for (position, entity) in &map.map {
            let tile = tiles.get(*entity).expect("Invalid entity");
            println!("{entity:?}: {position:?} {tile:?}");
        }
    }
}

fn handle_despawn_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut map: ResMut<TileMap>,
    tiles: Query<(Entity, &TilePosition), With<Tile>>,
) {
    if input.just_released(KeyCode::KeyD) {
        for (entity, position) in &tiles {
            map.map.remove(position).unwrap();
            commands.entity(entity).despawn();
        }
    }
}

fn handle_save_input(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();

    // Using DebugPipeline as the argument for save/load, we can save locally with JSON.

    if keys.just_released(KeyCode::Enter) {
        // Save every tile individually.
        for position in world.resource::<TileMap>().map.keys() {
            world
                .save(TilePipeline::new(*position))
                .expect("Failed to save");
        }
    } else if keys.just_released(KeyCode::Backspace) {
        // For ease of implementation, let's just load the origin.
        world
            .load(TilePipeline::new(TilePosition { x: 0, y: 0 }))
            .expect("Failed to load");
    }
}

pub struct TilePipeline {
    position: TilePosition,
    key: String,
}

impl TilePipeline {
    pub fn new(position: TilePosition) -> Self {
        Self {
            position,
            key: format!("examples/saves/pipeline/{}.{}", position.x, position.y),
        }
    }
}

impl Pipeline for TilePipeline {
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        &self.key
    }

    fn capture_seed(&self, builder: SnapshotBuilder) -> Snapshot {
        let world = builder.world();

        builder
            .extract_entity(
                *world
                    .resource::<TileMap>()
                    .map
                    .get(&self.position)
                    .expect("Could not find tile"),
            )
            .build()
    }

    fn apply_seed(&self, world: &mut World, snapshot: &Snapshot) -> Result<(), bevy_save::Error> {
        let mut mapper = EntityHashMap::default();

        world.resource_scope(|world, mut tiles: Mut<TileMap>| {
            for saved in &snapshot.entities {
                if let Some(existing) = tiles.map.get(&self.position) {
                    mapper.insert(saved.entity, *existing);
                } else {
                    let new = world.spawn_empty().id();
                    mapper.insert(saved.entity, new);
                    tiles.map.insert(self.position, new);
                }
            }
        });

        snapshot.applier(world).entity_map(&mut mapper).apply()
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
        // Register our types
        .register_type::<TileMap>()
        .register_type::<TilePosition>()
        .register_type::<Tile>()
        // Bevy's reflection requires we register each generic instance of a type individually
        // Note that we only need to register it in the AppTypeRegistry and not in the SaveableRegistry
        .register_type::<HashMap<TilePosition, Entity>>()
        // Resources
        .init_resource::<TileMap>()
        // Systems
        .add_systems(Startup, setup_world)
        .add_systems(
            Update,
            (display_world, handle_despawn_input, handle_save_input),
        )
        .run();
}
