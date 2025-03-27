//! Save directory management, can be used independently.

use std::{
    path::PathBuf,
    sync::LazyLock,
};

use platform_dirs::AppDirs;

include!(concat!(env!("OUT_DIR"), "/workspace.rs"));

/// The platform-specific save directory for the app.
///
/// [`WORKSPACE`] is the name of your project's workspace (parent folder) name.
///
/// | Windows                                             | Linux/*BSD                       | MacOS                                           |
/// |-----------------------------------------------------|----------------------------------|-------------------------------------------------|
/// | `C:\Users\%USERNAME%\AppData\Local\WORKSPACE\saves` | `~/.local/share/WORKSPACE/saves` | `~/Library/Application Support/WORKSPACE/saves` |
pub static SAVE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    AppDirs::new(Some(WORKSPACE), true)
        .unwrap()
        .data_dir
        .join("saves")
});

/// Returns the absolute path to a save file given its name.
pub fn get_save_file<K: std::fmt::Display>(key: K) -> PathBuf {
    SAVE_DIR.join(format!("{key}"))
}
