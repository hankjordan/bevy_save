# Bevy_save

[![][img_bevy]][bevy] [![][img_version]][crates] [![][img_doc]][doc] [![][img_license]][license] [![][img_tracking]][tracking] [![][img_downloads]][crates]

A framework for saving and loading game state in Bevy.

<https://user-images.githubusercontent.com/29737477/234151375-4c561c53-a8f4-4bfe-a5e7-b69af883bf65.mp4>

## Features

### Save file management

`bevy_save` automatically uses your app's workspace name to create a unique, permanent save location in the correct place for [whatever platform](#platforms) it is running on.

- `World::save()` and `World::load()` uses your app's save location to save and load your game state, handling all serialization and deserialization for you.
- These methods accept a `Pipeline`, a strongly typed representation of how you are going to be saving and loading.
- The `Pipeline` trait uses the `Backend` trait as an interface between disk / database and `bevy_save`.
- The `Backend` trait uses the `Format` trait to determine what format should be used in the actual save files (MessagePack / RON / JSON / etc)
  - The default `Pipeline` uses the `FileIO` backend which saves each snapshot to an individual file on the disk by the given key.
    - Many games have different requirements like saving to multiple directories, to a database, or to WebStorage.
    - You can use a different `Backend` by implementing your own `Pipeline` with a custom `Backend`.
  - The default `Pipeline` is set up to use `rmp_serde` as the file format.
    - You can use to a different `Format` by implementing your own `Pipeline` with a custom `Format`.

#### Save directory location

With the default `FileIO` backend, your save directory is managed for you.

`WORKSPACE` is the name of your project's workspace (parent folder) name.

| Windows                                             | Linux/\*BSD                      | MacOS                                           |
| --------------------------------------------------- | -------------------------------- | ----------------------------------------------- |
| `C:\Users\%USERNAME%\AppData\Local\WORKSPACE\saves` | `~/.local/share/WORKSPACE/saves` | `~/Library/Application Support/WORKSPACE/saves` |

On WASM, snapshots are saved to `LocalStorage`, with the key:

```ignore
WORKSPACE.KEY
```

### Snapshots and Rollback

`bevy_save` is not just about save files, it is about total control over game state.

This crate introduces a snapshot type which may be used directly:

- `Snapshot` is a serializable snapshot of all saveable resources, entities, and components.

Or via the `World` extension methods:

- `World::snapshot()` captures a snapshot of the current game state, including resources.
- `World::checkpoint()` captures a snapshot for later rollback / rollforward.
- `World::rollback()` rolls the game state backwards or forwards through any checkpoints you have created.

The `Rollbacks` resource also gives you fine-tuned control of the currently stored rollbacks.

### Type registration

`bevy_save` adds methods to Bevy's `App` for registering types that should be saved.
As long as the type implements `Reflect`, it can be registered and used with `bevy_save`.

- `App.init_pipeline::<P>()` initializes a `Pipeline` for use with save / load.
- `App.allow_rollback::<T>()` allows a type to roll back.
- `App.deny_rollback::<T>()` denies a type from rolling back.

### Type filtering

`bevy_save` allows you to explicitly filter types when creating a snapshot.

### Entity mapping

As Entity ids are not intended to be used as unique identifiers, `bevy_save` supports mapping Entity ids.

First, you'll need to get a `SnapshotApplier`:

- `Snapshot::applier()`
- `SnapshotApplier::new()`

The `SnapshotApplier` will then allow you to configure the entity map (and other settings) before applying:

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

`bevy_save` also supports `MapEntities` via reflection to allow you to update entity ids within components and resources.

See [Bevy's Parent Component](https://github.com/bevyengine/bevy/blob/v0.12.1/crates/bevy_hierarchy/src/components/parent.rs) for a simple example.

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

### Partial Snapshots

While `bevy_save` aims to make it as easy as possible to save your entire world, some games also need to be able to save only parts of the world.

`Builder` allows you to manually create snapshots like `DynamicSceneBuilder`:

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

Additionally, explicit type filtering like `Applier` is available when building snapshots:

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

### Pipeline

Pipelines allow you to use multiple different configurations of `Backend` and `Format` in the same `App`.

Pipelines also let you re-use `Snapshot` appliers and extractors.

## License

`bevy_save` is dual-licensed under MIT and Apache-2.0.

## Feature Flags

| Feature flag  | Description                             | Default? |
| ------------- | --------------------------------------- | -------- |
| `bevy_asset`  | Enables `bevy_asset` type registration  | Yes      |
| `bevy_render` | Enables `bevy_render` type registration | Yes      |
| `bevy_sprite` | Enables `bevy_sprite` type registration | Yes      |
| `brotli`      | Enables `Brotli` compression middleware | No       |

## Compatibility

### Bevy

NOTE: We do not track Bevy main.

| Bevy Version | Crate Version                     |
| ------------ | --------------------------------- |
| `0.13`       | `0.14`                            |
| `0.12`       | `0.10`, `0.11`, `0.12`, `0.13`    |
| `0.11`       | `0.9`                             |
| `0.10`       | `0.4`, `0.5`, `0.6`, `0.7`, `0.8` |
| `0.9`        | `0.1`, `0.2`, `0.3`               |

### Platforms

| Platform | Support            |
| -------- | ------------------ |
| Windows  | :heavy_check_mark: |
| MacOS    | :heavy_check_mark: |
| Linux    | :heavy_check_mark: |
| WASM     | :heavy_check_mark: |
| Android  | :question:         |
| iOS      | :question:         |

:heavy_check_mark: = First Class Support
—
:ok: = Best Effort Support
—
:zap: = Untested, but should work
—
:question: = Untested, probably won't work
—
:hammer_and_wrench: = In progress

[img_bevy]: https://img.shields.io/badge/Bevy-0.13-blue
[img_version]: https://img.shields.io/crates/v/bevy_save.svg
[img_doc]: https://docs.rs/bevy_save/badge.svg
[img_license]: https://img.shields.io/badge/license-MIT%2FApache-blue.svg
[img_downloads]: https://img.shields.io/crates/d/bevy_save.svg
[img_tracking]: https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue
[bevy]: https://crates.io/crates/bevy/0.13.0
[crates]: https://crates.io/crates/bevy_save
[doc]: https://docs.rs/bevy_save
[license]: https://github.com/hankjordan/bevy_save#license
[tracking]: https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking
