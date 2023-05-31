use std::{
    fs::File,
    io::{
        BufReader,
        BufWriter,
        Read,
        Write,
    },
};

use bevy::prelude::*;

use crate::{
    get_save_file,
    OwnedReader,
    OwnedWriter,
    SaveableError,
};

/// [`Read`] and [`Write`] interface used for save file storage.
pub trait Backend: Send + Sync + 'static {
    /// The backend's reader.
    type Reader: Read;

    /// The backend's writer.
    type Writer: Write;

    /// Attempts to open a reader for the save with the given name.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    fn reader(name: &str) -> Result<Self::Reader, SaveableError>;

    /// Attempts to open a writer for the save with the given name.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    fn writer(name: &str) -> Result<Self::Writer, SaveableError>;
}

/// Type-erased version of [`Backend`].
pub trait ErasedBackend: Send + Sync + 'static {
    /// Type-erased version of [`Backend::reader`]
    ///
    /// # Errors
    /// - See [`Backend::reader`]
    fn reader(&self, name: &str) -> Result<OwnedReader, SaveableError>;

    /// Type-erased version of [`Backend::writer`]
    ///
    /// # Errors
    /// - See [`Backend::writer`]
    fn writer(&self, name: &str) -> Result<OwnedWriter, SaveableError>;
}

impl<T> ErasedBackend for T
where
    T: Backend + Send + Sync + 'static,
{
    fn reader(&self, name: &str) -> Result<OwnedReader, SaveableError> {
        T::reader(name).map(|r| Box::new(r).into())
    }

    fn writer(&self, name: &str) -> Result<OwnedWriter, SaveableError> {
        T::writer(name).map(|w| Box::new(w).into())
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    /// Simple filesystem backend.
    ///
    /// Each name corresponds to an individual file on the disk.
    ///
    /// Files are stored in `SAVE_DIR`.
    pub struct FileIO;

    impl Backend for FileIO {
        type Reader = BufReader<File>;
        type Writer = BufWriter<File>;

        fn reader(name: &str) -> Result<Self::Reader, SaveableError> {
            let path = get_save_file(name);
            let file = File::open(path).map_err(SaveableError::other)?;

            Ok(BufReader::new(file))
        }

        fn writer(name: &str) -> Result<Self::Writer, SaveableError> {
            let path = get_save_file(name);
            let dir = path.parent().expect("Invalid save directory");

            std::fs::create_dir_all(dir).map_err(SaveableError::other)?;

            let file = File::create(path).map_err(SaveableError::other)?;

            Ok(BufWriter::new(file))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::FileIO;

// TODO
/*
mod wasm {
    use web_sys::Storage;

    #[allow(clippy::wildcard_imports)]
    use super::*;

    /// Simple `WebStorage` backend.
    pub struct WebStorage;

    pub struct WebReader {
        value: String,
    }

    impl Read for WebReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            todo!()
        }
    }

    pub struct WebWriter {
        storage: Storage,
        key: String,
        value: String,
    }

    impl Write for WebWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            todo!()
        }

        fn flush(&mut self) -> std::io::Result<()> {
            todo!()
        }
    }

    impl Backend for WebStorage {
        type Reader = WebReader;
        type Writer = WebWriter;

        fn reader(name: &str) -> Result<Self::Reader, SaveableError> {
            let storage = web_sys::window()
                .expect("No window")
                .local_storage()
                .expect("Failed to get local storage")
                .expect("No local storage");

            let value = storage.get_item(name)
                .expect("Failed to get value")
                .expect("No value");

            Ok(WebReader {
                value
            })
        }

        fn writer(name: &str) -> Result<Self::Writer, SaveableError> {
            let storage = web_sys::window()
                .expect("No window")
                .local_storage()
                .expect("Failed to get local storage")
                .expect("No local storage");

            Ok(WebWriter {
                storage,
                key: name.to_owned(),
                value: String::new(),
            })
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::WebStorage;
*/

/// The App's [`Backend`].
///
/// `bevy_save` will use this as the interface for saving and loading snapshots.
#[derive(Resource)]
pub struct AppBackend(Box<dyn ErasedBackend>);

impl AppBackend {
    /// Create a new [`AppBackend`] from the given [`Backend`].
    pub fn new<B: Backend>(backend: B) -> Self {
        Self(Box::new(backend))
    }

    /// Override the current [`Backend`].
    pub fn set<B: Backend>(&mut self, backend: B) {
        self.0 = Box::new(backend);
    }

    /// Attempts to open a reader for the save with the given name.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    pub fn reader(&self, name: &str) -> Result<OwnedReader, SaveableError> {
        self.0.reader(name)
    }

    /// Attempts to open a writer for the save with the given name.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    pub fn writer(&self, name: &str) -> Result<OwnedWriter, SaveableError> {
        self.0.writer(name)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for AppBackend {
    fn default() -> Self {
        Self(Box::new(FileIO))
    }
}
