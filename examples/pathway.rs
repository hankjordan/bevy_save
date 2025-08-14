//! An example of how to implement your own `Pipeline`.

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use bevy_save::prelude::*;
use bevy_save_macros::FlowLabel;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, Default, Resource, Serialize, Deserialize)]
pub struct TileMap {
    map: HashMap<TilePosition, Entity>,
}

#[derive(Clone, Copy, Component, Debug, Default, Serialize, Deserialize)]
pub enum Tile {
    #[default]
    Empty,
    Grass,
    Dirt,
    Stone,
    IronOre,
}

#[derive(Clone, Copy, Component, Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
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
    commands.spawn(Camera2d);
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

    if keys.just_released(KeyCode::Enter) {
        info!("Saving data");

        // Save every tile individually.
        let positions = world
            .resource::<TileMap>()
            .map
            .keys()
            .copied()
            .collect::<Vec<_>>();

        for position in positions {
            world
                .save(&TilePathway::new(position))
                .expect("Failed to save");
        }
    } else if keys.just_released(KeyCode::Backspace) {
        info!("Loading data");

        // For ease of implementation, let's just load the origin.
        world
            .load(&TilePathway::new(TilePosition { x: 0, y: 0 }))
            .expect("Failed to load");
    }
}

#[derive(Serialize, Deserialize)]
pub struct Capture {
    position: TilePosition,
    data: Option<Tile>,
}

impl CaptureInput<TilePathway> for Capture {
    type Builder = Capture;

    fn builder(pathway: &TilePathway, _world: &World) -> Self::Builder {
        Capture {
            position: pathway.position,
            data: None,
        }
    }

    fn build(builder: Self::Builder, _pathway: &TilePathway, _world: &World) -> Self {
        builder
    }
}

impl CaptureOutput<TilePathway> for Capture {
    type Builder = Capture;
    type Error = ();

    fn builder(self, _pathway: &TilePathway, _world: &mut World) -> Self::Builder {
        self
    }

    fn build(
        builder: Self::Builder,
        _pathway: &TilePathway,
        _world: &mut World,
    ) -> Result<Self, Self::Error> {
        Ok(builder)
    }
}

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
pub struct CaptureFlow;

fn capture_tile(In(mut cap): In<Capture>, map: Res<TileMap>, tiles: Query<&Tile>) -> Capture {
    cap.data = map
        .map
        .get(&cap.position)
        .and_then(|e| tiles.get(*e).ok())
        .copied();

    cap
}

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
pub struct ApplyFlow;

fn apply_tile(In(cap): In<Capture>, mut commands: Commands, mut map: ResMut<TileMap>) -> Capture {
    if let Some(data) = cap.data {
        let entity = *map
            .map
            .entry(cap.position)
            .or_insert_with(|| commands.spawn_empty().id());

        commands.entity(entity).insert((cap.position, data));
    } else if let Some(old) = map.map.remove(&cap.position) {
        if let Ok(mut cmds) = commands.get_entity(old) {
            cmds.try_despawn();
        }
    }

    cap
}

pub struct TilePathway {
    position: TilePosition,
}

impl TilePathway {
    pub fn new(position: TilePosition) -> Self {
        Self { position }
    }
}

impl Pathway for TilePathway {
    type Capture = Capture;
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = String;

    fn key(&self) -> Self::Key<'_> {
        format!(
            "examples/saves/pathway/{}.{}",
            self.position.x, self.position.y
        )
    }

    fn capture(&self, _world: &World) -> impl bevy_save::prelude::FlowLabel {
        CaptureFlow
    }

    fn apply(&self, _world: &World) -> impl bevy_save::prelude::FlowLabel {
        ApplyFlow
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
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            // Bevy Save
            SavePlugins,
        ))
        // Resources
        .init_resource::<TileMap>()
        // Pathway
        .init_pathway::<TilePathway>()
        // Flows
        .add_flows(CaptureFlow, capture_tile)
        .add_flows(ApplyFlow, apply_tile)
        // Systems
        .add_systems(Startup, setup_world)
        .add_systems(
            Update,
            (display_world, handle_despawn_input, handle_save_input),
        )
        .run();
}
