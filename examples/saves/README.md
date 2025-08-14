# Saves

While the examples save to this local folder, using the `DefaultBackend` (equivalent to `FileIO` on desktop) instead of the `DefaultDebugBackend` with your `Pathway` or `Pipeline` will save to a [managed, application-specific save directory](https://docs.rs/bevy_save/latest/bevy_save/prelude/static.SAVE_DIR.html) in the expected location for your platform.
