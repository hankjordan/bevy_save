# Bevy_save

[![][img_bevy]][bevy] [![][img_version]][crates] [![][img_doc]][doc] [![][img_license]][license] [![][img_tracking]][tracking] [![][img_downloads]][crates]

A framework for saving and loading application state in Bevy.

<https://user-images.githubusercontent.com/29737477/234151375-4c561c53-a8f4-4bfe-a5e7-b69af883bf65.mp4>

## Features

### Save file management

`bevy_save` automatically uses your app's workspace name to create a unique, permanent save location in the correct place for [any platform](#platforms) it can run on.

By default, [`World::save()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldPathwayExt.html#tymethod.save) and [`World::load()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldPathwayExt.html#tymethod.load) uses the managed save file location to save and load your application state, handling all serialization and deserialization for you.

#### Save directory location

With the default [`FileIO`] backend, your save directory is managed for you.

[`WORKSPACE`] is the name of your project's workspace (parent folder) name.

| Windows                                             | Linux/\*BSD                      | MacOS                                           |
| --------------------------------------------------- | -------------------------------- | ----------------------------------------------- |
| `C:\Users\%USERNAME%\AppData\Local\WORKSPACE\saves` | `~/.local/share/WORKSPACE/saves` | `~/Library/Application Support/WORKSPACE/saves` |

On WASM, snapshots are saved to [`LocalStorage`], with the key `WORKSPACE.KEY`.

### Reflection-based Snapshots

`bevy_save` is not just about save files, it is about total control over application state.

With the `"reflect"` feature enabled, this crate introduces a snapshot type which may be used directly:

- [`Snapshot`] is a serializable snapshot of all saveable resources, entities, and components.

Or via the [`World`] extension method [`WorldPathwayExt`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldPathwayExt.html) and [`WorldCheckpointExt`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldCheckpointExt.html):

- [`World::capture()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldPathwayExt.html#tymethod.capture) captures a snapshot of the current application state, including resources.
- [`World::apply()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldPathwayExt.html#tymethod.apply) applies a snapshot to the [`World`].

#### Versioning and Migrations

Applications can change over the history of development. Users expect that saves created in an older version will continue to work in newer versions.

`bevy_save` provides support for reflection-based migrations with the [`Migrate`] trait:

```rust
use bevy::prelude::*;
use bevy_save::prelude::*;

// `#[reflect(Migrate)]` registers `ReflectMigrate` with the `TypeRegistry`
// This allows `Snapshot`s to save the type version and apply migrations automatically
#[derive(Reflect, Component, Debug, PartialEq)]
#[type_path = "migrate"]
#[type_name = "Position"]
#[reflect(Component, Migrate)]
struct Position {
    xyz: (f32, f32, f32),
}

// The `Migrate` trait allows you to define a `Migrator`
// which will step the upgrade through each version
impl Migrate for Position {
    fn migrator() -> Migrator<Self> {
        #[derive(Reflect)]
        #[type_path = "migrate"]
        #[type_name = "Pos"]
        struct PosV0_1 {
            x: f32,
            y: f32,
        }

        Migrator::new::<PosV0_1>("0.1.0")
            .version("0.2.0", |v1| {
                // Changing type paths and type names is supported
                #[derive(Reflect)]
                #[type_path = "migrate"]
                #[type_name = "Position"]
                struct PosV0_2 {
                    x: f32,
                    y: f32,
                }

                Some(PosV0_2 { x: v1.x, y: v1.y })
            })
            .version("0.3.0", |v2| {
                #[derive(Reflect)]
                #[type_path = "migrate"]
                #[type_name = "Position"]
                struct PosV0_3 {
                    x: f32,
                    y: f32,
                    z: f32,
                }

                // Fields can be re-mapped from version to version, added, or removed
                Some(PosV0_3 {
                    x: v2.x,
                    y: v2.y,
                    z: 0.0,
                })
            })
            // The final version will represent the current layout
            .version("0.4.0", |v2| {
                Some(Self {
                    xyz: (v2.x, v2.y, v2.z),
                })
            })
    }
}
```

#### Rollbacks and checkpoints

With the `"checkpoints"` feature enabled, this crate provides methods for creating checkpoints which are ordered and can be rolled back / forwards through.

- [`World::checkpoint()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldCheckpointExt.html#tymethod.checkpoint) captures a snapshot for later rollback / rollforward.
- [`World::rollback()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldCheckpointExt.html#tymethod.rollback) rolls the application state backwards or forwards through any checkpoints you have created.

The [`Checkpoints`] resource also gives you fine-tuned control of the currently stored rollback checkpoints.

#### Type registration

No special traits or NewTypes necessary, `bevy_save` takes full advantage of Bevy's built-in reflection.
As long as the type implements [`Reflect`] and it's not filtered out, it will automatically be included in [`Snapshot`]s.

```rust
use bevy::prelude::*;
use bevy_save::prelude::*;

/// `#[reflect(Ignore)]` prevents the type from being included in any snapshots or checkpoints.
#[derive(Component, Reflect)]
#[reflect(Component, Ignore)]
struct NotIncluded {
    x: f32,
    y: f32,
    z: f32,
}

/// `#[reflect(IgnoreCheckpoint)]` prevents the type from being included in checkpoints (but they will still be in snapshots).
#[derive(Component, Reflect)]
#[reflect(Component, IgnoreCheckpoint)]
struct NotInCheckpoints {
    x: f32,
    y: f32,
    z: f32,
}
```

#### Pipeline

The [`Pipeline`] trait allows you to use multiple different configurations of [`Backend`] and [`Format`] in the same [`App`].

Using [`Pipeline`] also lets you re-use [`Snapshot`] appliers and builders.

```rust
use bevy::prelude::*;
use bevy_save::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Head;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Player;

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
```

#### Type Filtering and Partial Snapshots

While `bevy_save` aims to make it as easy as possible to save your entire world, some applications also need to be able to save only parts of the world.

[`Builder`] allows you to manually create snapshots like [`DynamicSceneBuilder`]:

```rust
use bevy::prelude::*;
use bevy_save::prelude::*;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct FancyMap {
    map: Vec<(u32, String)>,
}

fn build_snapshot(world: &World, target: Entity, children: Query<&Children>) -> Snapshot {
    Snapshot::builder(world)
        // Extract all resources
        .extract_all_resources()

        // Extract all descendants of `target`
        // This will include all components not denied by the builder's filter
        .extract_entities(children.iter_descendants(target))

        // Entities without any components will also be extracted
        // You can use `clear_empty` to remove them
        .clear_empty()

        // Build the `Snapshot`
        .build()
}

/// You are also able to extract resources by type
fn build_snapshot_resource(world: &World) -> Snapshot {
    Snapshot::builder(world)
        // Extract the resource by the type
        .extract_resource::<FancyMap>()

        // Build the `Snapshot`
        // It will only contain the one resource we extracted
        .build()
}

// Additionally, explicit type filtering like `ApplierRef` is available when building snapshots
fn build_snapshot_filter(world: &World) -> Snapshot {
    Snapshot::builder(world)
        // Exclude `Transform` from this `Snapshot`
        .deny::<Transform>()

        // Extract all matching entities and resources
        .extract_all()

        // Clear all extracted entities without any components
        .clear_empty()

        // Build the `Snapshot`
        .build()
}
```

#### Entity hooks

You are also able to add hooks when applying snapshots, similar to `bevy-scene-hook`.

This can be used for many things, like spawning the snapshot as a child of an entity:

```rust
use bevy::prelude::*;
use bevy_save::prelude::*;

fn apply_snapshot(world: &mut World, root: Entity, snapshot: Snapshot) {
    snapshot
        .applier(world)

        // This will be run for every Entity in the snapshot
        // It runs after the Entity's Components are loaded
        .hook(move |entity, cmds| {
            // You can use the hook to add, get, or remove Components
            if !entity.contains::<ChildOf>() {
                cmds.insert(ChildOf(root));
            }
        })

        .apply();
}
```

Hooks may also despawn entities:

```rust
use bevy::prelude::*;
use bevy_save::prelude::*;

fn apply_snapshot(world: &mut World, snapshot: Snapshot) {
    snapshot
        .applier(world)

        .hook(|entity, cmds| {
            if entity.contains::<Transform>() {
                cmds.despawn();
            }
        })

        // Alternatively, you could do the following:
        // .despawn::<With<Transform>>()

        .apply();
}
```

#### Entity mapping

As Entity ids are not intended to be used as unique identifiers, `bevy_save` supports mapping Entity ids.

This is done transparently by utilizing [`MapEntities`].

```rust
use bevy::{ecs::{entity::MapEntities, reflect::ReflectMapEntities}, prelude::*};
use bevy_save::prelude::*;
use std::collections::HashMap;

/// It is required to register `#[reflect(MapEntities)]`
#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
struct ExampleMap {
    data: HashMap<u32, Entity>,
}

impl MapEntities for ExampleMap {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.data.values_mut().for_each(|e| {
            *e = entity_mapper.get_mapped(*e);
        });
    }
}

/// While `bevy_save` supports entity maps, it is not necessary even if you are using `MapEntities`.
fn apply_snapshot(world: &mut World, snapshot: Snapshot) {
    snapshot
        .applier(world)

        // An entity map may be desired if you are already doing something like using persistent uuid entity identifiers
        // .entity_map(entity_map)

        // Otherwise, you should just despawn existing entities
        .despawn::<With<ExampleMap>>()

        .apply();
}
```

### Flows

When creating a complex application, snapshot builder and applier functions tend to get complex and unwieldy.

[`Flow`]s are chains of systems used to modularize this process, allowing you to build snapshots and apply them in stages.

They are defined similar to Bevy systems, but they require an input and an output.

Additionally, the introduction of [`Flow`]s allows reflection to become optional - bring your own serialization if you so wish!

```rust
use bevy::prelude::*;
use bevy_save::prelude::*;
use serde::{Serialize, Deserialize};

// Flow labels don't encode any behavior by themselves, only point to flows
#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
pub struct CaptureFlow;

fn main() {
    App::new()
        .add_flows(CaptureFlow, capture_tiles);
}

#[derive(Component, Clone, Serialize, Deserialize)]
enum Tile {
    Air,
    Dirt,
    Stone,
}

// User-defined captures make reflection unnecessary
#[derive(Serialize, Deserialize)]
struct MyCapture {
    // ... but then you'll need to specify everything you extract
    tiles: Vec<(u64, Tile)>,
}

// Flow systems have full access to the ECS (even write access)
fn capture_tiles(In(mut cap): In<MyCapture>, tiles: Query<(Entity, &Tile)>) -> MyCapture {
    cap.tiles.extend(tiles.iter().map(|(e, t)| (e.to_bits(), t.clone())));
    cap
}

// Flow systems can be added to flows from anywhere in your application
struct PluginA;

impl Plugin for PluginA {
    fn build(&self, app: &mut App) {
        app.add_flows(CaptureFlow, capture_tiles);
    }
}
```

#### Pathway

[`Pathway`] is the more flexible version of [`Pipeline`] which allows you to specify your own capture type and use [`Flow`]s.

```rust
use bevy::prelude::*;
use bevy_save::prelude::*;

// Pathways look very similar to pipelines, but there are a few key differences
pub struct SimplePathway;

impl Pathway for SimplePathway {
    // The capture type allows you to save anything you want to disk, even without using reflection
    type Capture = Snapshot;

    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;
    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "examples/saves/flows"
    }

    // Instead of capturing and applying directly, now these methods just return labels to user-defined flows
    // This allows for better dependency injection and reduces code complexity
    fn capture(&self, _world: &World) -> impl FlowLabel {
        CaptureFlow
    }

    fn apply(&self, _world: &World) -> impl FlowLabel {
        ApplyFlow
    }
}

// Flow labels don't encode any behavior by themselves, only point to flows
#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
pub struct CaptureFlow;

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
pub struct ApplyFlow;
```

### Backend

The [`Backend`] is the interface between your application and persistent storage.

Some example backends may include [`FileIO`], sqlite, [`LocalStorage`], or network storage.

```rust
use bevy_save::prelude::*;
use serde::{de::DeserializeSeed, Serialize};
use smol::{fs::{File, create_dir_all}, io::{AsyncReadExt, AsyncWriteExt}};

struct FileIO;

impl<K: std::fmt::Display + Send> Backend<K> for FileIO {
    async fn save<F: Format, T: Serialize>(&self, key: K, value: &T) -> Result<(), Error> {
        let path = get_save_file(format!("{key}{}", F::extension()));
        let dir = path.parent().expect("Invalid save directory");
        create_dir_all(dir).await?;
        let mut buf = Vec::new();
        F::serialize(&mut buf, value)?;
        let mut file = File::create(path).await?;
        Ok(file.write_all(&buf).await?)
    }

    async fn load<F: Format, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
        &self,
        key: K,
        seed: S,
    ) -> Result<T, Error> {
        let path = get_save_file(format!("{key}{}", F::extension()));
        let mut file = File::open(path).await?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;
        F::deserialize(&*buf, seed)
    }
}
```

### Format

[`Format`] is how your application serializes and deserializes your data.

[`Format`]s can either be human-readable like [`JSON`] or binary like [`MessagePack`].

```rust
use bevy_save::prelude::*;
use io_adapters::WriteExtension;
use serde::{de::DeserializeSeed, Serialize};
use std::io::{Read, Write};

pub struct RONFormat;

impl Format for RONFormat {
    fn extension() -> &'static str {
        ".ron"
    }

    fn serialize<W: Write, T: Serialize>(writer: W, value: &T) -> Result<(), Error> {
        let mut ser = ron::Serializer::new(
            writer.write_adapter(),
            Some(ron::ser::PrettyConfig::default()),
        )
        .map_err(Error::saving)?;

        value.serialize(&mut ser).map_err(Error::saving)
    }

    fn deserialize<R: Read, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
        reader: R,
        seed: S,
    ) -> Result<T, Error> {
        ron::options::Options::default()
            .from_reader_seed(reader, seed)
            .map_err(Error::loading)
    }
}
```

### Prefabs

The [`Prefab`] trait allows you to easily spawn entities from a blueprint.

```rust
use bevy::prelude::*;
use bevy_save::prelude::*;

const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
const BALL_DIAMETER: f32 = 30.;
const BALL_SPEED: f32 = 400.0;
const BALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Ball;

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
struct Velocity(Vec2);

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
```

## Stability

`bevy_save` attempts to provide stability guarantees for [`Snapshot`] serialization and deserialization between crate versions on a best-effort basis, enforced with unit tests.

If a breaking change is introduced, **the number after the `+` on the crate version will be incremented** and it will be supported via the `version` method on [`SnapshotDeserializer`].

`bevy_save` relies on serialization to create save files and as such is exposed to internal implementation details for types.
As a result, Bevy or other crate updates may break your save file format.
It should be possible to mitigate this by defining [`ReflectMigrate`] for any offending types.

Changing what entities have what components or how you use your entities or resources in your logic can also result in broken saves.

### Entity

> For all intents and purposes, [`Entity`] should be treated as an opaque identifier. The internal bit representation is liable to change from release to release as are the behaviors or performance characteristics of any of its trait implementations (i.e. `Ord`, `Hash,` etc.). This means that changes in [`Entity`]’s representation, though made readable through various functions on the type, are not considered breaking changes under SemVer.
>
> In particular, directly serializing with `Serialize` and `Deserialize` make zero guarantee of long term wire format compatibility. Changes in behavior will cause serialized [`Entity`] values persisted to long term storage (i.e. disk, databases, etc.) will fail to deserialize upon being updated.
>
> — [Bevy's `Entity` documentation](https://docs.rs/bevy/latest/bevy/ecs/entity/struct.Entity.html#stability-warning)

`bevy_save` serializes and deserializes entities directly. If you need to maintain compatibility across Bevy versions, consider adding a unique identifier [`Component`] to your tracked entities.

## Compatibility

`bevy_save` follows [Semantic Versioning 2.0.0](https://semver.org/), with additional metadata: `MAJOR.MINOR.PATCH+SNAPSHOT`

- **Major**: Breaking API changes and/or [`Snapshot`] format changes
- **Minor**: Backwards-compatible API changes
- **Patch**: Backwards-compatible bug-fixes
- **Snapshot**: [`Snapshot`] version, incremented when the wire format of [`Snapshot`] changes in a way that will break existing applications

### Bevy

| Bevy Version              | Crate Version                                                    |
| ------------------------- | ---------------------------------------------------------------- |
| `0.16`                    | `0.18+3`, `0.19+3`, `1.0+4`<sup> [4](#4)</sup>, `2.0+4`, `3.0+4` |
| `0.15`                    | `0.16+3`<sup> [3](#3)</sup>, `0.17+3`                            |
| `0.14`<sup> [2](#2)</sup> | `0.15+2`                                                         |
| `0.13`                    | `0.14+1`                                                         |
| `0.12`                    | `0.10+1`, `0.11+1`, `0.12+1`, `0.13+1`                           |
| `0.11`                    | `0.9+1`                                                          |
| `0.10`                    | `0.4+0`, `0.5+0`, `0.6+1`<sup> [1](#1)</sup>, `0.7+1`, `0.8+1`   |
| `0.9`                     | `0.1`, `0.2+0`<sup> [0](#0)</sup>, `0.3+0`                       |

#### Snapshot Version

0. <a id="0"></a> `bevy_save` introduced serialization support
1. <a id="1"></a> `bevy_save` introduced a new [`Snapshot`] format
2. <a id="2"></a> `bevy` changed [`Entity`]'s on-disk representation
3. <a id="3"></a> `bevy_save` began using [`FromReflect`] when taking snapshots
4. <a id="4"></a> `bevy_save` introduced a new [`Snapshot`] format, see below

### Migrating

<details>
<summary>0.18+3 -> 0.19+3</summary>

This version introduced [`Pathway`], which is effectively a superset of [`Pipeline`].

- In `World::capture`, `World:apply`, `World::save`, `World::load` methods and similar, add a `&` before your existing pipeline
- Previously provided `Commands` extension traits and associated commands have been removed (since [`Pathway`] operates on references), you'll need to write your own or use events instead
- If you're using `default-features = false`, you'll need to add the `reflect` and `checkpoints` features in order to get parity with the last version
- `SnapshotBuilder` and `SnapshotApplier` have been renamed to [`BuilderRef`] and [`ApplierRef`], respectively.

</details>

<details>
<summary>0.19+3 -> 1.0+4</summary>

This version introduced versioning and migrations.

- Removed the `checkpoints` field from [`Snapshot`], instead saving [`Checkpoints`] as a resource via [`Reflect`].
- `SnapshotVersion::V3` can be used with the `version` method on [`SnapshotDeserializer`] to load a snapshot created in a previous version (since `0.16`) if the snapshot had checkpoints.
- Snapshots created in a previous version without checkpoints should load as expected.
- The fields for all serializers and deserializers have been made private. Use the `new` methods to construct them.
- Non self-describing formats such as `postcard` should now work as expected.
- Deserialization of [`Snapshot`] will fail for this version if `PartialReflect::reflect_clone` is not implemented for all contained types. While this is typically automatically implemented, opaque types must now manually add the attribute `#[reflect(Clone)]` in order for `Snapshot::from_reflect` to succeed.

</details>

<details>
<summary>1.0+4 -> 2.0+4</summary>

This version made some ergonomic changes and fixed a few bugs.

- `Applier` now respects `ReflectMapEntities` for `Component`s.
- `BoxedPartialReflect` is now `DynamicValue`.
- The `CloneReflect` trait has been removed. [`Snapshot`], [`Checkpoints`], and all the wrapper types now implement `Clone`.
- `WorldCheckpointExt::rollback_with` has been removed, use `WorldCheckpointExt::rollback` instead.
- Deserialization of [`Snapshot`] **will no longer fail** if `PartialReflect::reflect_clone` is not implemented for any contained types.
</details>

<details open>
<summary>2.0+4 -> 3.0+4 (latest)</summary>

This version improved support for `Relationship`s and introduced `ReflectIgnore` and `ReflectIgnoreCheckpoint`.

- Types with `Relationship`s must be registered to act correctly. Register `#[reflect(Relationship)]` and `#[reflect(RelationshipTarget)]` accordingly.
- Both the `Relationship` and `RelationshipTarget` types must be registered. A warning will be logged if this is not the case.
- If registered, `RelationshipTarget` types will no longer be included in [`Snapshot`] or applied on load. The `Relationship` type is the "source of truth" and the "target" entities will be populated from those values.
- `CheckpointRegistry` has been removed. Instead, register `#[reflect(IgnoreCheckpoint)]`.
- `SavePlugin` is now `SaveCorePlugin` to differentiate from `SavePlugins`.

</details>

### Platforms

| Platform | Support |
| -------- | ------- |
| Windows  | Yes     |
| MacOS    | Yes     |
| Linux    | Yes     |
| WASM     | Yes     |
| Android  | No      |
| iOS      | No      |

## Feature Flags

| Feature flag  | Description                             | Default? |
| ------------- | --------------------------------------- | -------- |
| `reflect`     | Enables reflection-based snapshots      | Yes      |
| `checkpoints` | Enables reflection-based checkpoints    | Yes      |
| `log`         | Enables logging of warnings             | Yes      |
| `bevy_asset`  | Enables `bevy_asset` type registration  | Yes      |
| `bevy_render` | Enables `bevy_render` type registration | Yes      |
| `bevy_sprite` | Enables `bevy_sprite` type registration | Yes      |
| `brotli`      | Enables `Brotli` compression middleware | No       |

## License

`bevy_save` is dual-licensed under MIT and Apache-2.0.

[img_bevy]: https://img.shields.io/badge/Bevy-0.16-blue
[img_version]: https://img.shields.io/crates/v/bevy_save.svg
[img_doc]: https://docs.rs/bevy_save/badge.svg
[img_license]: https://img.shields.io/badge/license-MIT%2FApache-blue.svg
[img_downloads]: https://img.shields.io/crates/d/bevy_save.svg
[img_tracking]: https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue
[bevy]: https://crates.io/crates/bevy/0.16.1
[crates]: https://crates.io/crates/bevy_save
[doc]: https://docs.rs/bevy_save
[license]: https://github.com/hankjordan/bevy_save#license
[tracking]: https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking
[`Snapshot`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.Snapshot.html
[`Builder`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.Builder.html
[`BuilderRef`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.BuilderRef.html
[`ApplierRef`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.ApplierRef.html
[`Checkpoints`]: https://docs.rs/bevy_save/latest/bevy_save/checkpoint/struct.Checkpoints.html
[`Pipeline`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.Pipeline.html
[`Backend`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.Backend.html
[`Format`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.Format.html
[`FileIO`]: https://docs.rs/bevy_save/latest/bevy_save/backend/struct.FileIO.html
[`JSON`]: https://docs.rs/bevy_save/latest/bevy_save/format/struct.JSONFormat.html
[`MessagePack`]: https://docs.rs/bevy_save/latest/bevy_save/format/struct.RMPFormat.html
[`Prefab`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.Prefab.html
[`Flow`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.Flow.html
[`Pathway`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.Pathway.html
[`SnapshotDeserializer`]: https://docs.rs/bevy_save/latest/bevy_save/reflect/struct.SnapshotDeserializer.html
[`Migrate`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.Migrate.html
[`Migrator`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.Migrator.html
[`ReflectMigrate`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.ReflectMigrate.html
[`WORKSPACE`]: https://docs.rs/bevy_save/latest/bevy_save/dir/constant.WORKSPACE.html
[`App`]: https://docs.rs/bevy/latest/bevy/prelude/struct.App.html
[`Component`]: https://docs.rs/bevy/latest/bevy/prelude/trait.Component.html
[`DynamicSceneBuilder`]: https://docs.rs/bevy/latest/bevy/prelude/struct.DynamicSceneBuilder.html
[`Entity`]: https://docs.rs/bevy/latest/bevy/prelude/struct.Entity.html
[`FromReflect`]: https://docs.rs/bevy/latest/bevy/prelude/trait.FromReflect.html
[`Reflect`]: https://docs.rs/bevy/latest/bevy/prelude/trait.Reflect.html
[`ReflectDeserialize`]: https://docs.rs/bevy/latest/bevy/prelude/struct.ReflectDeserialize.html
[`World`]: https://docs.rs/bevy/latest/bevy/prelude/struct.World.html
[`MapEntities`]: https://docs.rs/bevy/latest/bevy/ecs/entity/trait.MapEntities.html
[`LocalStorage`]: https://docs.rs/web-sys/latest/web_sys/struct.Storage.html
