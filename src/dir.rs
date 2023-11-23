use std::path::PathBuf;

use lazy_static::lazy_static;
use platform_dirs::AppDirs;

include!(concat!(env!("OUT_DIR"), "/workspace.rs"));

lazy_static! {
    /// The platform-specific save directory for the app.
    ///
    /// `WORKSPACE` is the name of your project's workspace (parent folder) name.
    ///
    /// | Windows                                             | Linux/*BSD                       | MacOS                                           |
    /// |-----------------------------------------------------|----------------------------------|-------------------------------------------------|
    /// | `C:\Users\%USERNAME%\AppData\Local\WORKSPACE\saves` | `~/.local/share/WORKSPACE/saves` | `~/Library/Application Support/WORKSPACE/saves` |
    pub static ref SAVE_DIR: PathBuf = {
        AppDirs::new(Some(WORKSPACE), true)
            .unwrap()
            .data_dir
            .join("saves")
    };
}

/// Returns the absolute path to a save file given its name.
pub fn get_save_file<K: std::fmt::Display>(key: K) -> PathBuf {
    SAVE_DIR.join(format!("{key}"))
}
