//! A simplified implementation of the classic game "Breakout".
//! Modified to demonstrate integration of `bevy_save`.

use bevy::{
    ecs::system::{
        SystemParam,
        SystemState,
    },
    math::bounding::{
        Aabb2d,
        BoundingCircle,
        BoundingVolume,
        IntersectsVolume,
    },
    prelude::*,
    sprite::{
        MaterialMesh2dBundle,
        Mesh2dHandle,
    },
    utils::Instant,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_save::prelude::*;

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const PADDLE_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
const PADDLE_SPEED: f32 = 500.0;
// How close can the paddle get to the wall
const PADDLE_PADDING: f32 = 10.0;

// We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
const BALL_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);
const BALL_DIAMETER: f32 = 30.;
const BALL_SPEED: f32 = 400.0;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
// These values are exact
const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
const GAP_BETWEEN_BRICKS: f32 = 5.0;
// These values are lower bounds, as the number of bricks is computed
const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const HELP_FONT_SIZE: f32 = 16.0;
const HELP_TEXT_PADDING: Val = Val::Px(5.0);

const TOAST_FONT_SIZE: f32 = 32.0;
const TOAST_TEXT_PADDING: Val = Val::Px(5.0);
const TOAST_TEXT_COLOR: Color = Color::rgb(0.5, 0.1, 0.1);
const TOAST_DURATION: f32 = 0.5;

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const BRICK_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

#[rustfmt::skip]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            file_path: "examples/assets".to_owned(),
            ..default()
        }))
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        // Add our gameplay simulation systems to the fixed timestep schedule
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                move_paddle,
                check_for_collisions,
                play_collision_sound,
            )
                // `chain`ing systems together runs them in order
                .chain(),
        )
        .add_systems(Update, (update_scoreboard, bevy::window::close_on_esc))

        .add_plugins((
            // Inspector
            WorldInspectorPlugin::new(),

            // Bevy Save
            SavePlugins,
        ))

        // Register our types
        .register_type::<Paddle>()
        .register_type::<Ball>()
        .register_type::<Velocity>()
        .register_type::<Collider>()
        .register_type::<Brick>()
        .register_type::<Scoreboard>()
        .register_type::<ScoreboardUi>()
        .register_type::<Toast>()

        // Setup
        .add_systems(Startup, (setup_help, setup_entity_count).after(setup))
        
        // Systems
        .add_systems(Update, (update_entity_count, handle_save_input, update_toasts))

        .run();
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Paddle;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut, Reflect, Default)]
#[reflect(Component)]
struct Velocity(Vec2);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Brick;

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

/// Which side of the arena is this wall located on?
enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

// This resource tracks the game's score
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct Scoreboard {
    score: usize,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct ScoreboardUi;

// Add the game's entities to our world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, paddle_y, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle,
        Collider,
    ));

    // Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
            ..default()
        },
        Ball,
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
    ));

    // Scoreboard
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new("Score: ", TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: TEXT_COLOR,
                ..default()
            }),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
        ScoreboardUi,
    ));

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

    // Bricks
    let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
    let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

    assert!(total_width_of_bricks > 0.0);
    assert!(total_height_of_bricks > 0.0);

    // Given the space available, compute how many rows and columns of bricks we can fit
    let n_columns = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_vertical_gaps = n_columns - 1;

    // Because we need to round the number of columns,
    // the space on the top and sides of the bricks only captures a lower bound, not an exact value
    let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0;
    let left_edge_of_bricks = center_of_bricks
        // Space taken up by the bricks
        - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
        // Space taken up by the gaps
        - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

    // In Bevy, the `translation` of an entity describes the center point,
    // not its bottom-left corner
    let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.;
    let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
            );

            // brick
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: BRICK_COLOR,
                        ..default()
                    },
                    transform: Transform {
                        translation: brick_position.extend(0.0),
                        scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Brick,
                Collider,
            ));
        }
    }
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    // Calculate the new horizontal paddle position based on player input
    let new_paddle_position =
        paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_seconds();

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text, With<ScoreboardUi>>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

fn check_for_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();

    // check collision with walls
    for (collider_entity, transform, maybe_brick) in &collider_query {
        let collision = collide_with_side(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
            Aabb2d::new(
                transform.translation.truncate(),
                transform.scale.truncate() / 2.,
            ),
        );

        if let Some(collision) = collision {
            // Sends a collision event so that other systems can react to the collision
            collision_events.send_default();

            // Bricks should be despawned and increment the scoreboard on collision
            if maybe_brick.is_some() {
                scoreboard.score += 1;
                commands.entity(collider_entity).despawn();
            }

            // reflect the ball when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // only reflect if the ball's velocity is going in the opposite direction of the
            // collision
            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
            }

            // reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            // reflect velocity on the y-axis if we hit something on the y-axis
            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    sound: Res<CollisionSound>,
) {
    // Play a sound once per frame if a collision occurred.
    if !collision_events.is_empty() {
        // This prevents events staying active on the next frame.
        collision_events.clear();
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            // auto-despawn the entity when playback finishes
            settings: PlaybackSettings::DESPAWN,
        });
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

// Returns `Some` if `ball` collides with `wall`. The returned `Collision` is the
// side of `wall` that `ball` hit.
fn collide_with_side(ball: BoundingCircle, wall: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&wall) {
        return None;
    }

    let closest = wall.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn setup_help(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new("Enter - Save | Backspace - Load\n", TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: HELP_FONT_SIZE,
                color: TEXT_COLOR,
            }),
            TextSection::new(
                "Space - Checkpoint | A - Rollback | D - Rollforward\n",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: HELP_FONT_SIZE,
                    color: TEXT_COLOR,
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: HELP_TEXT_PADDING,
            left: HELP_TEXT_PADDING,
            ..default()
        }),
    );
}

#[derive(Component)]
struct EntityCount;

fn setup_entity_count(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TextBundle::from_section("", TextStyle {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: HELP_FONT_SIZE,
            color: TEXT_COLOR,
        })
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: HELP_TEXT_PADDING,
            right: HELP_TEXT_PADDING,
            ..default()
        }),
        EntityCount,
    ));
}

fn update_entity_count(entities: Query<Entity>, mut counters: Query<&mut Text, With<EntityCount>>) {
    let mut text = counters.single_mut();
    text.sections[0].value = format!("Entities: {:?}", entities.iter().len());
}

#[derive(Component, Reflect)]
pub struct Toast {
    time: Instant,
}

impl Default for Toast {
    fn default() -> Self {
        Self {
            time: Instant::now(),
        }
    }
}

fn update_toasts(mut commands: Commands, toasts: Query<(Entity, &Toast)>) {
    for (entity, toast) in &toasts {
        if toast.time.elapsed().as_secs_f32() > TOAST_DURATION {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(SystemParam)]
struct Toaster<'w, 's> {
    commands: Commands<'w, 's>,
    asset_server: Res<'w, AssetServer>,
    toasts: Query<'w, 's, Entity, With<Toast>>,
}

impl<'w, 's> Toaster<'w, 's> {
    fn show(&mut self, text: &str) {
        for entity in &self.toasts {
            self.commands.entity(entity).despawn_recursive();
        }

        self.commands.spawn((
            TextBundle::from_sections([TextSection::new(text, TextStyle {
                font: self.asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: TOAST_FONT_SIZE,
                color: TOAST_TEXT_COLOR,
            })])
            .with_style(Style {
                position_type: PositionType::Absolute,
                bottom: TOAST_TEXT_PADDING,
                right: TOAST_TEXT_PADDING,
                ..default()
            }),
            Toast::default(),
        ));
    }
}

struct BreakoutPipeline;

impl Pipeline for BreakoutPipeline {
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "examples/saves/breakout"
    }

    fn capture(builder: SnapshotBuilder) -> Snapshot {
        builder
            .deny::<Mesh2dHandle>()
            .deny::<Handle<ColorMaterial>>()
            .extract_entities_matching(|e| {
                e.contains::<Paddle>() || e.contains::<Ball>() || e.contains::<Brick>()
            })
            .extract_resource::<Scoreboard>()
            .extract_rollbacks()
            .build()
    }

    fn apply(world: &mut World, snapshot: &Snapshot) -> Result<(), bevy_save::Error> {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = Mesh2dHandle(meshes.add(Circle::default()));
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let material = materials.add(ColorMaterial::from(BALL_COLOR));

        snapshot
            .applier(world)
            .despawn::<Or<(With<Paddle>, With<Ball>, With<Brick>)>>()
            .hook(move |entity, cmd| {
                if entity.contains::<Ball>() {
                    cmd.insert((mesh.clone(), material.clone()));
                }
            })
            .apply()
    }
}

fn handle_save_input(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();

    let mut text = None;

    if keys.just_released(KeyCode::Space) {
        world.checkpoint::<BreakoutPipeline>();
        text = Some("Checkpoint");
    } else if keys.just_released(KeyCode::Enter) {
        world.save(BreakoutPipeline).expect("Failed to save");
        text = Some("Save");
    } else if keys.just_released(KeyCode::Backspace) {
        world.load(BreakoutPipeline).expect("Failed to load");
        text = Some("Load");
    } else if keys.just_released(KeyCode::KeyA) {
        world
            .rollback::<BreakoutPipeline>(1)
            .expect("Failed to rollback");
        text = Some("Rollback");
    } else if keys.just_released(KeyCode::KeyD) {
        world
            .rollback::<BreakoutPipeline>(-1)
            .expect("Failed to rollforward");
        text = Some("Rollforward");
    }

    if let Some(text) = text {
        let mut state: SystemState<Toaster> = SystemState::new(world);
        let mut toaster = state.get_mut(world);

        info!(text);
        toaster.show(text);

        state.apply(world);
    }
}
