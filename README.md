# Bevy_save

[![][img_bevy]][bevy] [![][img_version]][crates] [![][img_doc]][doc] [![][img_license]][license] [![][img_tracking]][tracking] [![][img_downloads]][crates]

A framework for saving and loading application state in Bevy.

<https://user-images.githubusercontent.com/29737477/234151375-4c561c53-a8f4-4bfe-a5e7-b69af883bf65.mp4>

## Features

### Save file management

`bevy_save` automatically uses your app's workspace name to create a unique, permanent save location in the correct place for [any platform](#platforms) it can run on.

By default, [`World::save()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldSaveableExt.html#tymethod.save) and [`World::load()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldSaveableExt.html#tymethod.load) uses the managed save file location to save and load your application state, handling all serialization and deserialization for you.

#### Save directory location

With the default [`FileIO`] backend, your save directory is managed for you.

[`WORKSPACE`] is the name of your project's workspace (parent folder) name.

| Windows                                             | Linux/\*BSD                      | MacOS                                           |
| --------------------------------------------------- | -------------------------------- | ----------------------------------------------- |
| `C:\Users\%USERNAME%\AppData\Local\WORKSPACE\saves` | `~/.local/share/WORKSPACE/saves` | `~/Library/Application Support/WORKSPACE/saves` |

On WASM, snapshots are saved to [`LocalStorage`], with the key `WORKSPACE.KEY`.

### Snapshots and rollback

`bevy_save` is not just about save files, it is about total control over application state.

This crate introduces a snapshot type which may be used directly:

- [`Snapshot`] is a serializable snapshot of all saveable resources, entities, and components.

Or via the [`World`] extension methods [`WorldSaveableExt`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldSaveableExt.html) and [`WorldCheckpointExt`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldCheckpointExt.html):

- [`World::snapshot()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldSaveableExt.html#tymethod.snapshot) captures a snapshot of the current application state, including resources.
- [`World::checkpoint()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldCheckpointExt.html#tymethod.checkpoint) captures a snapshot for later rollback / rollforward.
- [`World::rollback()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.WorldCheckpointExt.html#tymethod.rollback) rolls the application state backwards or forwards through any checkpoints you have created.

The [`Checkpoints`] resource also gives you fine-tuned control of the currently stored rollback checkpoints.

### Type registration

No special traits or NewTypes necessary, `bevy_save` takes full advantage of Bevy's built-in reflection.
As long as the type implements [`Reflect`], it can be registered and used with `bevy_save`.

`bevy_save` provides extension traits for [`App`] allowing you to do so.

- [`App.init_pipeline::<P>()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.AppSaveableExt.html#tymethod.init_pipeline) initializes a [`Pipeline`] for use with save / load.
- [`App.allow_checkpoint::<T>()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.AppCheckpointExt.html#tymethod.allow_checkpoint) allows a type to roll back.
- [`App.deny_checkpoint::<T>()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.AppCheckpointExt.html#tymethod.deny_checkpoint) denies a type from rolling back.

### Backend

The [`Backend`] is the interface between your application and persistent storage.

Some example backends may include [`FileIO`], sqlite, [`LocalStorage`], or network storage.

```rust,ignore
#[derive(Default, Resource)]
pub struct FileIO;

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

```rust,ignore
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

### Pipeline

The [`Pipeline`] trait allows you to use multiple different configurations of [`Backend`] and [`Format`] in the same [`App`].

Using [`Pipeline`] also lets you re-use [`Snapshot`] appliers and builders.

```rust,ignore
struct HeirarchyPipeline;

impl Pipeline for HeirarchyPipeline {
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "examples/saves/heirarchy"
    }

    fn capture(&self, builder: SnapshotBuilder) -> Snapshot {
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

### Prefabs

The [`Prefab`] trait allows you to easily spawn entities from a blueprint.

```rust,ignore
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Ball;

#[derive(Reflect)]
struct BallPrefab {
    position: Vec3,
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
            Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
        ));
    }

    fn extract(builder: SnapshotBuilder) -> SnapshotBuilder {
        // We don't actually need to save all of those runtime components.
        // Only save the translation of the Ball.
        builder.extract_prefab(|entity| {
            Some(BallPrefab {
                position: entity.get::<Transform>()?.translation,
            })
        })
    }
}
```

### Type Filtering and Partial Snapshots

While `bevy_save` aims to make it as easy as possible to save your entire world, some applications also need to be able to save only parts of the world.

[`SnapshotBuilder`] allows you to manually create snapshots like [`DynamicSceneBuilder`]:

```rust,ignore
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
```

You are also able to extract resources by type:

```rust,ignore
Snapshot::builder(world)
    // Extract the resource by the type name
    // In this case, we extract the resource from the `manual` example
    .extract_resource::<FancyMap>()

    // Build the `Snapshot`
    // It will only contain the one resource we extracted
    .build()
```

Additionally, explicit type filtering like [`SnapshotApplier`] is available when building snapshots:

```rust,ignore
Snapshot::builder(world)
    // Exclude `Transform` from this `Snapshot`
    .deny::<Transform>()

    // Extract all matching entities and resources
    .extract_all()

    // Clear all extracted entities without any components
    .clear_empty()

    // Build the `Snapshot`
    .build()
```

### Entity hooks

You are also able to add hooks when applying snapshots, similar to `bevy-scene-hook`.

This can be used for many things, like spawning the snapshot as a child of an entity:

```rust,ignore
let snapshot = Snapshot::from_world(world);

snapshot
    .applier(world)

    // This will be run for every Entity in the snapshot
    // It runs after the Entity's Components are loaded
    .hook(move |entity, cmds| {
        // You can use the hook to add, get, or remove Components
        if !entity.contains::<Parent>() {
            cmds.set_parent(parent);
        }
    })

    .apply();
```

Hooks may also despawn entities:

```rust,ignore
let snapshot = Snapshot::from_world(world);

snapshot
    .applier(world)

    .hook(|entity, cmds| {
        if entity.contains::<A>() {
            cmds.despawn();
        }
    })
```

### Entity mapping

As Entity ids are not intended to be used as unique identifiers, `bevy_save` supports mapping Entity ids.

First, you'll need to get a [`SnapshotApplier`]:

- [`Snapshot::applier()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.Snapshot.html#method.applier)
- [`SnapshotApplier::new()`](https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.SnapshotApplier.html#method.new)

The [`SnapshotApplier`] will then allow you to configure the entity map (and other settings) before applying:

```rust,ignore
let snapshot = Snapshot::from_world(world);

snapshot
    .applier(world)

    // Your entity map
    .entity_map(HashMap::default())

    // Despawn all entities matching (With<A>, Without<B>)
    .despawn::<(With<A>, Without<B>)>()

    .apply();
```

#### MapEntities

`bevy_save` also supports [`MapEntities`](https://docs.rs/bevy/latest/bevy/ecs/entity/trait.MapEntities.html) via reflection to allow you to update entity ids within components and resources.

See [Bevy's Parent Component](https://github.com/bevyengine/bevy/blob/v0.15.3/crates/bevy_hierarchy/src/components/parent.rs) for a simple example.

## Stability warning

`bevy_save` does not _yet_ provide any stability guarantees for save file format between crate versions.

`bevy_save` relies on serialization to create save files and as such is exposed to internal implementation details for types.
Expect Bevy or other crate updates to break your save file format.
It should be possible to mitigate this by overriding [`ReflectDeserialize`] for any offending types.

Changing what entities have what components or how you use your entities or resources in your logic can also result in broken saves.
While `bevy_save` does not _yet_ have explicit support for save file migration, you can use [`SnapshotApplier::hook`](https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.SnapshotApplier.html#method.hook) to account for changes while applying a snapshot.

If your application has specific migration requirements, please [open an issue](https://github.com/hankjordan/bevy_save/issues/new).

### Entity

> For all intents and purposes, [`Entity`] should be treated as an opaque identifier. The internal bit representation is liable to change from release to release as are the behaviors or performance characteristics of any of its trait implementations (i.e. `Ord`, `Hash,` etc.). This means that changes in [`Entity`]’s representation, though made readable through various functions on the type, are not considered breaking changes under SemVer.
>
> In particular, directly serializing with `Serialize` and `Deserialize` make zero guarantee of long term wire format compatibility. Changes in behavior will cause serialized [`Entity`] values persisted to long term storage (i.e. disk, databases, etc.) will fail to deserialize upon being updated.
>
> — [Bevy's `Entity` documentation](https://docs.rs/bevy/latest/bevy/ecs/entity/struct.Entity.html#stability-warning)

`bevy_save` serializes and deserializes entities directly. If you need to maintain compatibility across Bevy versions, consider adding a unique identifier [`Component`] to your tracked entities.

### Stabilization

`bevy_save` will become a candidate for stabilization once [all stabilization tasks](https://github.com/hankjordan/bevy_save/milestone/2) have been completed.

## Compatibility

### Bevy

| Bevy Version              | Crate Version                     |
| ------------------------- | --------------------------------- |
| `0.16`                    | `0.18`                            |
| `0.15`                    | `0.16`<sup> [2](#2)</sup>, `0.17` |
| `0.14`<sup> [1](#1)</sup> | `0.15`                            |
| `0.13`                    | `0.14`                            |
| `0.12`                    | `0.10`, `0.11`, `0.12`, `0.13`    |
| `0.11`                    | `0.9`                             |
| `0.10`                    | `0.4`, `0.5`, `0.6`, `0.7`, `0.8` |
| `0.9`                     | `0.1`, `0.2`, `0.3`               |

#### Save format changes (since `0.15`)

1. <a id="1"></a> `bevy` changed [`Entity`]'s on-disk representation
2. <a id="2"></a> `bevy_save` began using [`FromReflect`] when taking snapshots

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
[`SnapshotBuilder`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.SnapshotBuilder.html
[`SnapshotApplier`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/struct.SnapshotApplier.html
[`Checkpoints`]: https://docs.rs/bevy_save/latest/bevy_save/checkpoint/struct.Checkpoints.html
[`Pipeline`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.Pipeline.html
[`Backend`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.Backend.html
[`Format`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.Format.html
[`FileIO`]: https://docs.rs/bevy_save/latest/bevy_save/backend/struct.FileIO.html
[`JSON`]: https://docs.rs/bevy_save/latest/bevy_save/format/struct.JSONFormat.html
[`MessagePack`]: https://docs.rs/bevy_save/latest/bevy_save/format/struct.RMPFormat.html
[`Prefab`]: https://docs.rs/bevy_save/latest/bevy_save/prelude/trait.Prefab.html
[`WORKSPACE`]: https://docs.rs/bevy_save/latest/bevy_save/dir/constant.WORKSPACE.html
[`App`]: https://docs.rs/bevy/latest/bevy/prelude/struct.App.html
[`Component`]: https://docs.rs/bevy/latest/bevy/prelude/trait.Component.html
[`DynamicSceneBuilder`]: https://docs.rs/bevy/latest/bevy/prelude/struct.DynamicSceneBuilder.html
[`Entity`]: https://docs.rs/bevy/latest/bevy/prelude/struct.Entity.html
[`FromReflect`]: https://docs.rs/bevy/latest/bevy/prelude/trait.FromReflect.html
[`Reflect`]: https://docs.rs/bevy/latest/bevy/prelude/trait.Reflect.html
[`ReflectDeserialize`]: https://docs.rs/bevy/latest/bevy/prelude/struct.ReflectDeserialize.html
[`World`]: https://docs.rs/bevy/latest/bevy/prelude/struct.World.html
[`LocalStorage`]: https://docs.rs/web-sys/latest/web_sys/struct.Storage.html
