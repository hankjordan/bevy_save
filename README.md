# Bevy_save
[![][img_bevy]][bevy] [![][img_version]][crates] [![][img_doc]][doc] [![][img_license]][license] [![][img_tracking]][tracking] [![][img_downloads]][crates]

A framework for saving and loading game state in Bevy.

## Features

`bevy_save` is primarily built around extension traits to Bevy's `World`.

### Serialization and Deserialization

While Bevy's `DynamicScene` only allows you to save entities and components, `bevy_save` enables you to save everything, including resources.

- `World.serialize<S>()` and `World.deserialize<D>()` allow you to serialize and deserialize game state with your own serializer / deserializer.

### Save file management

`bevy_save` automatically uses your app's workspace name to create a unique, permanent save directory in the correct place for whatever platform it is running on.

Supports Windows, Linux, and MacOS.

- `World.save()` and `World.load()` uses your app's save directory to save and load your game state to disk, handling all serialization and deserialization for you.

### Snapshots and Rollback

`bevy_save` is not just about save files, it is about total control over game state. Rollback allows you to keep multiple snapshots of game state in memory and scroll through them in real time.

- `World.snapshot()` captures a snapshot of the current game state, including resources.
- `World.checkpoint()` captures a snapshot for later rollback / rollforward.
- `World.rollback()` rolls the game state backwards or forwards through any checkpoints you have created.

## License

`bevy_save` is dual-licensed under MIT and Apache-2.0.

## Compatibility

NOTE: We do not track Bevy main.

|Bevy Version|Crate Version          |
|------------|-----------------------|
|`0.9`       |`0.1`, `0.2`, `0.3`    |

[img_bevy]: https://img.shields.io/badge/Bevy-0.9-blue
[img_version]: https://img.shields.io/crates/v/bevy_save.svg
[img_doc]: https://docs.rs/bevy_save/badge.svg
[img_license]: https://img.shields.io/badge/license-MIT%2FApache-blue.svg
[img_downloads]:https://img.shields.io/crates/d/bevy_save.svg
[img_tracking]: https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue

[bevy]: https://crates.io/crates/bevy/0.9.1
[crates]: https://crates.io/crates/bevy_save
[doc]: https://docs.rs/bevy_save/
[license]: https://github.com/hankjordan/bevy_save#license
[tracking]: https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking
