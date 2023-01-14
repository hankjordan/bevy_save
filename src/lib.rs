#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![doc = include_str!("../README.md")]

use bevy::{
    ecs::entity::EntityMap,
    prelude::*,
    reflect::{
        GetTypeRegistration,
        TypeRegistration,
    },
};

use crate::reflect::CloneReflect;

/// Reflect: module for Reflect-related traits and impls
pub mod reflect;

/// The global registry of types that should be tracked by `bevy_save`
#[derive(Resource, Default)]
pub struct SaveableRegistry {
    types: Vec<TypeRegistration>,
}

/// The global registry of snapshots used for rollback / rollforward
#[derive(Resource, Default)]
pub struct Rollbacks {
    snapshots: Vec<Snapshot>,
    active: Option<usize>,
}

impl Rollbacks {
    /// Given a new `Snapshot`, insert it and set it as the currently active rollback.
    /// If you rollback and then insert a checkpoint, it will erase all rollforward snapshots.
    pub fn checkpoint(&mut self, snapshot: Snapshot) {
        let active = self.active.unwrap_or(0);

        self.snapshots.truncate(active + 1);

        self.snapshots.push(snapshot);

        self.active = Some(self.snapshots.len() - 1);
    }

    /// Rolls back the given number of checkpoints.
    /// If checkpoints is negative, it rolls forward.
    /// This function will always clamp itself to valid snapshots.
    /// Rolling back or further farther than what is valid will just return the oldest / newest snapshot.
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_sign_loss)]
    pub fn rollback(&mut self, checkpoints: isize) -> Option<&Snapshot> {
        if let Some(active) = self.active {
            let raw = active as isize - checkpoints;
            let new = raw.clamp(0, self.snapshots.len() as isize - 1) as usize;

            self.active = Some(new);
            Some(&self.snapshots[new])
        } else {
            None
        }
    }
}

/// A complete snapshot of the current World.
pub struct Snapshot {
    resources: Vec<Box<dyn Reflect>>,
    scene: DynamicScene,
}

impl Clone for Snapshot {
    fn clone(&self) -> Self {
        Self {
            resources: self.resources.clone_value(),
            scene: self.scene.clone_value(),
        }
    }
}

impl Snapshot {
    /// Apply the `Snapshot` to the `World`, restoring it to the saved state.
    pub fn apply(&self, world: &mut World) {
        world.clear_entities();
        world.clear_trackers();

        let _s = self.scene.write_to_world(world, &mut EntityMap::default());

        let registry_arc = world.resource::<AppTypeRegistry>().clone();
        let registry = registry_arc.read();

        for reflect in &self.resources {
            if let Some(reg) = registry.get(reflect.type_id()) {
                if let Some(res) = reg.data::<ReflectResource>() {
                    res.apply(world, reflect.as_reflect());
                }
            }
        }
    }
}

/// Extension trait that adds save-related methods to Bevy's `World`.
pub trait WorldSaveableExt {
    /// Returns a snapshot of the current World state.
    fn snapshot(&self) -> Snapshot;

    /// Creates a checkpoint for rollback.
    fn checkpoint(&mut self);

    /// Rolls back / forward the World state.
    fn rollback(&mut self, checkpoints: isize);
}

impl WorldSaveableExt for World {
    fn snapshot(&self) -> Snapshot {
        let registry = self.resource::<AppTypeRegistry>();
        let saveables = self.resource::<SaveableRegistry>();

        let mut resources = Vec::new();

        for reg in &saveables.types {
            if let Some(res) = reg.data::<ReflectResource>() {
                if let Some(reflect) = res.reflect(self) {
                    resources.push(reflect.clone_value());
                }
            }
        }

        let scene = DynamicScene::from_world(self, registry);

        Snapshot { resources, scene }
    }

    fn checkpoint(&mut self) {
        let snap = self.snapshot();

        let mut state = self.resource_mut::<Rollbacks>();

        state.checkpoint(snap);
    }

    fn rollback(&mut self, checkpoints: isize) {
        let mut state = self.resource_mut::<Rollbacks>();

        if let Some(snap) = state.rollback(checkpoints).cloned() {
            snap.apply(self);
        }
    }
}

/// Extension trait that adds save-related methods to Bevy's `App`.
pub trait AppSaveableExt {
    /// Register a type as saveable - it will be included in World snapshots and affected by save/load.
    fn register_saveable<T: 'static + GetTypeRegistration>(&mut self) -> &mut Self;
}

impl AppSaveableExt for App {
    fn register_saveable<T: 'static + GetTypeRegistration>(&mut self) -> &mut Self {
        self ////
            .init_resource::<Rollbacks>()
            .init_resource::<SaveableRegistry>()
            .register_type::<T>()
            .add_startup_system(register::<T>)
    }
}

fn register<T: 'static + GetTypeRegistration>(mut registry: ResMut<SaveableRegistry>) {
    registry.types.push(T::get_type_registration());
}

/// Prelude: convenient import for all the user-facing APIs provided by the crate
pub mod prelude {
    pub use crate::*;
}
