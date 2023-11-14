use bevy::prelude::*;

use crate::{
    prelude::*,
    Error,
};

pub trait Pipeline: Sized {
    type Backend: Backend<Self::Key> + Resource + Default;
    type Saver: Saver;
    type Loader: Loader;

    type Key;

    fn build(app: &mut App) {
        app.world.insert_resource(Self::Backend::default());
    }

    fn capture(&self, world: &World) -> Snapshot {
        Snapshot::from_world(world)
    }

    fn apply(world: &mut World, snapshot: Snapshot) -> Result<(), Error> {
        snapshot.apply(world)
    }

    fn key(self) -> Self::Key;

    fn save(self, world: &World) -> Result<(), Error> {
        let registry = world.resource::<AppTypeRegistry>();
        let backend = world.resource::<Self::Backend>();

        let snapshot = self.capture(world);

        let ser = SnapshotSerializer::new(&snapshot, registry);

        backend.save::<Self::Saver, _>(self.key(), ser)
    }

    fn load(self, world: &mut World) -> Result<(), Error> {
        let registry = world.resource::<AppTypeRegistry>().clone();
        let reg = registry.read();
        let backend = world.resource::<Self::Backend>();

        let de = SnapshotDeserializer::new(&reg);

        let snapshot = backend.load::<Self::Loader, _>(self.key(), de)?;

        Self::apply(world, snapshot)
    }
}

impl<'a> Pipeline for &'a str {
    type Backend = DefaultBackend;
    type Saver = DefaultSaver;
    type Loader = DefaultLoader;

    type Key = &'a str;

    fn key(self) -> Self::Key {
        self
    }
}

/// Uses `JSON` and saves to the given local path.
pub struct DebugPipeline<'a>(pub &'a str);

impl<'a> Pipeline for DebugPipeline<'a> {
    type Backend = DefaultDebugBackend;
    type Saver = DefaultDebugSaver;
    type Loader = DefaultDebugLoader;

    type Key = &'a str;

    fn key(self) -> Self::Key {
        self.0
    }
}
