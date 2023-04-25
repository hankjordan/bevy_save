# Bevy_save
[![][img_bevy]][bevy] [![][img_version]][crates] [![][img_doc]][doc] [![][img_license]][license] [![][img_tracking]][tracking] [![][img_downloads]][crates]

A framework for saving and loading game state in Bevy.

## Features

### Serialization and Deserialization

While Bevy's `DynamicScene` only allows you to save entities and components, `bevy_save` enables you to save everything, including resources.

- `World::serialize<S>()` and `World::deserialize<D>()` allow you to manually serialize and deserialize game state with your own serializer / deserializer.

### Save file management

`bevy_save` automatically uses your app's workspace name to create a unique, permanent save directory in the correct place for whatever platform it is running on.

Supports Windows, Linux, and MacOS. WASM support is in progress.

- `World::save()` and `World::load()` uses your app's save directory to save and load your game state to disk, handling all serialization and deserialization for you.
- These methods use the `AppSaver` and `AppLoader` resources to determine what save format to use.
- By default, this is set up to use `rmp_serde` for serialization and deserialization.
- However, is extremely easy to switch to a custom save file format, see `"examples/json.rs"` for how you can do this.

### Snapshots and Rollback

`bevy_save` is not just about save files, it is about total control over game state.

This crate introduces a few snapshot types which may be used directly:

- `Snapshot` is a serializable snapshot of all saveable resources, entities, and components.
- `Rollback` is a serializable snapshot of all saveable resources, entities, and components that are included in rollbacks.

Or via the `World` extension methods:

- `World::snapshot()` captures a snapshot of the current game state, including resources. (equivalent to `Snapshot::from_world()`)
- `World::checkpoint()` captures a snapshot for later rollback / rollforward.
- `World::rollback()` rolls the game state backwards or forwards through any checkpoints you have created.

The `Rollbacks` resource also gives you fine-tuned control of the currently stored rollbacks.

### Type registration

`bevy_save` adds methods to Bevy's `App` for registering types that should be saved. 
As long as the type implements `Reflect`, it can be registered and used with `bevy_save`.
**Types that are not explicitly registered in the `SaveableRegistry` are not included in save/load**.

- `App.register_saveable::<T>()` registers a type as saveable, allowing it to be included in saves and rollbacks.
- `App.ignore_rollback::<T>()` excludes a type from rollback.
- `App.allow_rollback::<T>()` allows you to re-include a type in rollback after it has already been set to ignore rollback.

### Type filtering

While types that are not registered with `SaveableRegistry` are automatically filtered out for you,
`bevy_save` also allows you to explicitly filter types when creating a snapshot.

- `Snapshot::from_world_with_filter()`
- `Rollback::from_world_with_filter()`

### Entity mapping

As Entity ids are not intended to be used as unique identifiers, `bevy_save` supports mapping Entity ids.

First, you'll need to get a `SnapshotApplier`:

- `World::deserialize_applier()` while manually deserializing a snapshot.
- `World::load_applier()` while loading from a named save.
- `World::rollback_applier()` while rolling back / forward.

Or directly on the snapshot types:

- `Snapshot::applier()`
- `Rollback::applier()`

The `SnapshotApplier` will then allow you to configure the `EntityMap` (and other settings) before applying:

```rust
let snapshot = Snapshot::from_world(world);

snapshot
    .applier(world)
    .map(EntityMap::default())
    .despawn(DespawnMode::default())
    .mapping(MappingMode::default())
    .apply();
```

Note that by default `bevy_save` snapshots do not behave like Bevy's `DynamicScene` when applying:

If you use the methods that do not return an `Applier` (`deserialize`, `load`, `rollback`, `apply`),
any entities not present in the snapshot are despawned and existing entities may be overridden with snapshot data.

Because of this, in many cases you will not even need an `EntityMap`.

#### MapEntities

`bevy_save` also supports `MapEntities` via reflection to allow you to update entity ids within components and resources.

See [Bevy's Parent Component](https://github.com/bevyengine/bevy/blob/v0.10.1/crates/bevy_hierarchy/src/components/parent.rs) for a simple example.

## License

`bevy_save` is dual-licensed under MIT and Apache-2.0.

## Compatibility

NOTE: We do not track Bevy main.

|Bevy Version|Crate Version          |
|------------|-----------------------|
|`0.10`      |`0.4`, `0.5`, `0.6`    |
|`0.9`       |`0.1`, `0.2`, `0.3`    |

[img_bevy]: https://img.shields.io/badge/Bevy-0.10-blue
[img_version]: https://img.shields.io/crates/v/bevy_save.svg
[img_doc]: https://docs.rs/bevy_save/badge.svg
[img_license]: https://img.shields.io/badge/license-MIT%2FApache-blue.svg
[img_downloads]:https://img.shields.io/crates/d/bevy_save.svg
[img_tracking]: https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue

[bevy]: https://crates.io/crates/bevy/0.10.1
[crates]: https://crates.io/crates/bevy_save
[doc]: https://docs.rs/bevy_save/
[license]: https://github.com/hankjordan/bevy_save#license
[tracking]: https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking
