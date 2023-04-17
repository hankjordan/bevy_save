#![allow(clippy::uninlined_format_args)]

use std::{
    env,
    fs,
    path::Path,
};

fn main() {
    let workspace = env::var("OUT_DIR")
        .ok()
        .map(|v| Path::new(&v).to_path_buf())
        .and_then(|path| {
            for ancestor in path.ancestors() {
                if let Some(last) = ancestor.file_name() {
                    if last == "target" {
                        return ancestor
                            .parent()
                            .and_then(|p| p.file_name())
                            .and_then(|p| p.to_str())
                            .map(|p| p.to_owned());
                    }
                }
            }

            None
        })
        .expect("Could not find parent workspace.");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("workspace.rs");

    fs::write(
        dest,
        format!(
            "/// The name of the project's workspace.\npub const WORKSPACE: &str = {:?};",
            workspace
        ),
    )
    .unwrap();
}
