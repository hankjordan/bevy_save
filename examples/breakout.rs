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
    platform::time::Instant,
    prelude::*,
};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use bevy_save::{
    prelude::*,
    reflect::checkpoint::Checkpoints,
};

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
const PADDLE_SPEED: f32 = 500.0;
// How close can the paddle get to the wall
const PADDLE_PADDING: f32 = 10.0;
const PADDLE_OFFSET: f32 = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

// We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
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

const SCOREBOARD_FONT_SIZE: f32 = 33.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
const BRICK_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const WALL_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            file_path: "examples/assets".to_owned(),
            ..default()
        }))
        .insert_resource(Score(0))
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
        .add_systems(Update, (update_scoreboard, close_on_esc))
        .add_plugins((
            // Inspector
            EguiPlugin::default(),
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
        .register_type::<Score>()
        .register_type::<ScoreboardUi>()
        .register_type::<Toast>()
        // Register prefabs
        .register_type::<PaddlePrefab>()
        .register_type::<BallPrefab>()
        .register_type::<BrickPrefab>()
        // Setup
        .add_systems(Startup, (setup_help, setup_entity_count).after(setup))
        // Systems
        .add_systems(
            Update,
            (update_entity_count, handle_save_input, update_toasts),
        )
        .run();
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Paddle;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Ball;

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
struct Velocity(Vec2);

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Brick;

#[derive(Resource, Deref)]
struct CollisionSound(Handle<AudioSource>);

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    sprite: Sprite,
    transform: Transform,
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
    /// Location of the *center* of the wall, used in `transform.translation()`
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    /// (x, y) dimensions of the wall, used in `transform.scale()`
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
            sprite: Sprite::from_color(WALL_COLOR, Vec2::ONE),
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
            collider: Collider,
        }
    }
}

// This resource tracks the game's score
#[derive(Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
struct Score(usize);

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ScoreboardUi;

#[derive(Reflect, Default)]
struct PaddlePrefab {
    position: f32,
}

impl Prefab for PaddlePrefab {
    type Marker = Paddle;

    fn spawn(self, target: Entity, world: &mut World) {
        world.entity_mut(target).insert((
            Sprite::from_color(PADDLE_COLOR, Vec2::ONE),
            Transform {
                translation: Vec3::new(self.position, PADDLE_OFFSET, 0.0),
                scale: PADDLE_SIZE.extend(1.0),
                ..default()
            },
            Collider,
        ));
    }

    fn extract(builder: BuilderRef) -> BuilderRef {
        builder.extract_prefab(|entity| {
            Some(PaddlePrefab {
                position: entity.get::<Transform>()?.translation.x,
            })
        })
    }
}

#[derive(Reflect)]
struct BallPrefab {
    position: Vec3,
    velocity: Vec2,
}

impl Default for BallPrefab {
    fn default() -> Self {
        Self {
            position: BALL_STARTING_POSITION,
            velocity: INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED,
        }
    }
}

impl Prefab for BallPrefab {
    type Marker = Ball;

    fn spawn(self, target: Entity, world: &mut World) {
        // Some entities will need initialization from world state, such as mesh assets.
        // We can do that here.
        let mesh = world.resource_mut::<Assets<Mesh>>().add(Circle::default());
        let material = world
            .resource_mut::<Assets<ColorMaterial>>()
            .add(BALL_COLOR);

        world.entity_mut(target).insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(self.position)
                .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.)),
            Ball,
            Velocity(self.velocity),
        ));
    }

    fn extract(builder: BuilderRef) -> BuilderRef {
        // We don't actually need to save all of those runtime components.
        // Only save the translation and velocity of the Ball.
        builder.extract_prefab(|entity| {
            Some(BallPrefab {
                position: entity.get::<Transform>()?.translation,
                velocity: entity.get::<Velocity>()?.0,
            })
        })
    }
}

#[derive(Reflect)]
struct BrickPrefab {
    position: Vec2,
}

impl Prefab for BrickPrefab {
    type Marker = Brick;

    fn spawn(self, target: Entity, world: &mut World) {
        world.entity_mut(target).insert((
            Sprite {
                color: BRICK_COLOR,
                ..default()
            },
            Transform {
                translation: self.position.extend(0.0),
                scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
                ..default()
            },
            Collider,
        ));
    }

    fn extract(builder: BuilderRef) -> BuilderRef {
        builder.extract_prefab(|entity| {
            let position = entity.get::<Transform>()?.translation;

            Some(BrickPrefab {
                position: Vec2::new(position.x, position.y),
            })
        })
    }
}

// Add the game's entities to our world
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2d);

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Paddle
    commands.spawn_prefab(PaddlePrefab::default());

    // Ball
    commands.spawn_prefab(BallPrefab::default());

    // Scoreboard
    commands
        .spawn((
            Text::new("Score: "),
            TextFont {
                font_size: SCOREBOARD_FONT_SIZE,
                ..default()
            },
            TextColor(TEXT_COLOR),
            ScoreboardUi,
            Node {
                position_type: PositionType::Absolute,
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextFont {
                font_size: SCOREBOARD_FONT_SIZE,
                ..default()
            },
            TextColor(SCORE_COLOR),
        ));

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

    // Bricks
    let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = PADDLE_OFFSET + GAP_BETWEEN_PADDLE_AND_BRICKS;
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
            let position = Vec2::new(
                offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
            );

            // brick
            commands.spawn_prefab(BrickPrefab { position });
        }
    }
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paddle_transform: Single<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    // Calculate the new horizontal paddle position based on player input
    let new_paddle_position =
        paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_secs();

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

fn update_scoreboard(
    score: Res<Score>,
    score_root: Single<Entity, (With<ScoreboardUi>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    *writer.text(*score_root, 1) = score.to_string();
}

fn check_for_collisions(
    mut commands: Commands,
    mut score: ResMut<Score>,
    ball_query: Single<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.into_inner();

    for (collider_entity, collider_transform, maybe_brick) in &collider_query {
        let collision = ball_collision(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
            Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.,
            ),
        );

        if let Some(collision) = collision {
            // Sends a collision event so that other systems can react to the collision
            collision_events.write_default();

            // Bricks should be despawned and increment the scoreboard on collision
            if maybe_brick.is_some() {
                commands.entity(collider_entity).despawn();
                **score += 1;
            }

            // Reflect the ball's velocity when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // Reflect only if the velocity is in the opposite direction of the collision
            // This prevents the ball from getting stuck inside the bar
            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
            }

            // Reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            // Reflect velocity on the y-axis if we hit something on the y-axis
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
        commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

// Returns `Some` if `ball` collides with `bounding_box`.
// The returned `Collision` is the side of `bounding_box` that `ball` hit.
fn ball_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(ball.center());
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

// Bevy_save

const HELP_FONT_SIZE: f32 = 16.0;
const HELP_TEXT_PADDING: Val = Val::Px(5.0);

const TOAST_FONT_SIZE: f32 = 32.0;
const TOAST_TEXT_PADDING: Val = Val::Px(5.0);
const TOAST_TEXT_COLOR: Color = Color::srgb(0.5, 0.1, 0.1);
const TOAST_DURATION: f32 = 0.5;

fn setup_help(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new(
            "Enter - Save | Backspace - Load\nSpace - Checkpoint | A - Rollback | D - Rollforward",
        ),
        TextFont {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: HELP_FONT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Node {
            position_type: PositionType::Absolute,
            bottom: HELP_TEXT_PADDING,
            left: HELP_TEXT_PADDING,
            ..default()
        },
    ));
}

#[derive(Component)]
struct EntityCount;

fn setup_entity_count(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: HELP_FONT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Node {
            position_type: PositionType::Absolute,
            top: HELP_TEXT_PADDING,
            right: HELP_TEXT_PADDING,
            ..default()
        },
        EntityCount,
    ));
}

fn update_entity_count(entities: Query<Entity>, mut counters: Query<&mut Text, With<EntityCount>>) {
    let mut text = counters.single_mut().unwrap();
    text.0 = format!("Entities: {:?}", entities.iter().len());
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

impl Toaster<'_, '_> {
    fn show(&mut self, text: &str) {
        for entity in &self.toasts {
            self.commands.entity(entity).despawn();
        }

        self.commands.spawn((
            Text::new(text),
            TextFont {
                font: self.asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: TOAST_FONT_SIZE,
                ..default()
            },
            TextColor(TOAST_TEXT_COLOR),
            Node {
                position_type: PositionType::Absolute,
                bottom: TOAST_TEXT_PADDING,
                right: TOAST_TEXT_PADDING,
                ..default()
            },
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

    fn capture(&self, builder: BuilderRef) -> Snapshot {
        builder
            .extract_all_prefabs::<PaddlePrefab>()
            .extract_all_prefabs::<BallPrefab>()
            .extract_all_prefabs::<BrickPrefab>()
            .extract_resource::<Score>()
            .extract_resource::<Checkpoints>()
            .build()
    }

    fn apply(&self, world: &mut World, snapshot: &Snapshot) -> Result<(), bevy_save::Error> {
        snapshot
            .applier(world)
            .despawn::<Or<(
                WithPrefab<PaddlePrefab>,
                WithPrefab<BallPrefab>,
                WithPrefab<BrickPrefab>,
            )>>()
            .prefab::<PaddlePrefab>()
            .prefab::<BallPrefab>()
            .prefab::<BrickPrefab>()
            .apply()
    }
}

fn handle_save_input(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();

    let mut text = None;

    if keys.just_released(KeyCode::Space) {
        world.checkpoint(&BreakoutPipeline);
        text = Some("Checkpoint");
    } else if keys.just_released(KeyCode::Enter) {
        world.save(&BreakoutPipeline).expect("Failed to save");
        text = Some("Save");
    } else if keys.just_released(KeyCode::Backspace) {
        world.load(&BreakoutPipeline).expect("Failed to load");
        text = Some("Load");
    } else if keys.just_released(KeyCode::KeyA) {
        world
            .rollback(&BreakoutPipeline, 1)
            .expect("Failed to rollback");
        text = Some("Rollback");
    } else if keys.just_released(KeyCode::KeyD) {
        world
            .rollback(&BreakoutPipeline, -1)
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

pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}
