use std::{
    collections::HashSet,
    marker::PhantomData,
    sync::Arc,
};

use bevy::{
    ecs::{
        entity::EntityMap,
        query::ReadOnlyWorldQuery,
        system::EntityCommands,
        world::EntityRef,
    },
    prelude::*,
};

/// A [`ReadOnlyWorldQuery`] filter.
pub trait Filter: Send + Sync {
    /// Collect all entities from the given [`World`] matching the filter.
    fn collect(&self, world: &mut World) -> HashSet<Entity>;
}

struct FilterMarker<F>(PhantomData<F>);

impl<F> Filter for FilterMarker<F>
where
    F: ReadOnlyWorldQuery + Send + Sync,
{
    fn collect(&self, world: &mut World) -> HashSet<Entity> {
        world
            .query_filtered::<Entity, F>()
            .iter(world)
            .collect::<HashSet<_>>()
    }
}

/// A boxed [`Filter`].
pub type BoxedFilter = Box<dyn Filter>;

impl dyn Filter {
    /// Create an opaque type that implements [`Filter`] from a [`ReadOnlyWorldQuery`].
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_save::prelude::*;
    /// #[derive(Component)]
    /// struct A;
    ///
    /// #[derive(Component)]
    /// struct B;
    ///
    /// let filter = <dyn Filter>::new::<(With<A>, Without<B>)>();
    /// ```
    pub fn new<F>() -> impl Filter
    where
        F: ReadOnlyWorldQuery + Send + Sync,
    {
        FilterMarker::<F>(PhantomData)
    }

    /// Create a [`BoxedFilter`] from a [`ReadOnlyWorldQuery`].
    pub fn boxed<F>() -> BoxedFilter
    where
        F: ReadOnlyWorldQuery + Send + Sync + 'static,
    {
        Box::new(Self::new::<F>())
    }
}

/// Determines what the snapshot will do to existing entities when applied.
#[derive(Default)]
pub enum DespawnMode {
    /// Despawn entities missing from the save
    ///
    /// `bevy_save` default
    #[default]
    Missing,

    /// Despawn entities missing from the save matching filter
    MissingWith(BoxedFilter),

    /// Despawn unmapped entities
    Unmapped,

    /// Despawn unmapped entities matching filter
    UnmappedWith(BoxedFilter),

    /// Despawn all entities
    ///
    /// This is probably not what you want - in most cases this will close your app's [`Window`]
    All,

    /// Despawn all entities matching filter
    AllWith(BoxedFilter),

    /// Keep all entities
    ///
    /// `bevy_scene` default
    None,
}

impl DespawnMode {
    /// Create a new instance of [`DespawnMode::UnmappedWith`] with the given filter.
    pub fn unmapped_with<F>() -> Self
    where
        F: ReadOnlyWorldQuery + Send + Sync + 'static,
    {
        DespawnMode::UnmappedWith(<dyn Filter>::boxed::<F>())
    }

    /// Create a new instance of [`DespawnMode::AllWith`] with the given filter.
    pub fn all_with<F>() -> Self
    where
        F: ReadOnlyWorldQuery + Send + Sync + 'static,
    {
        DespawnMode::UnmappedWith(<dyn Filter>::boxed::<F>())
    }
}

/// A [`Hook`] runs on each entity when applying a snapshot.
///
/// # Example
/// This could be used to apply entities as children of another entity.
/// ```
/// # use bevy::prelude::*;
/// # use bevy_save::prelude::*;
/// # let mut app = App::new();
/// # app.add_plugins(MinimalPlugins);
/// # app.add_plugins(SavePlugins);
/// # let world = &mut app.world;
/// # let snapshot = Snapshot::from_world(world);
/// # let parent = world.spawn_empty().id();
/// snapshot
///     .applier(world)
///     .hook(move |entity, cmds| {
///         if !entity.contains::<Parent>() {
///             cmds.set_parent(parent);
///         }
///     })
///     .apply();
/// ```
pub trait Hook: for<'a> Fn(&'a EntityRef, &'a mut EntityCommands) + Send + Sync {}

impl<T> Hook for T where T: for<'a> Fn(&'a EntityRef, &'a mut EntityCommands) + Send + Sync {}

/// A boxed [`Hook`].
pub type BoxedHook = Box<dyn Hook>;

/// Determines how the snapshot will map entities when applied.
#[derive(Default)]
pub enum MappingMode {
    /// If unmapped, attempt a one-to-one mapping. If that fails, spawn a new entity.
    ///
    /// `bevy_save` default
    #[default]
    Simple,

    /// If unmapped, spawn a new entity.
    ///
    /// `bevy_scene` default
    Strict,
}

/// The App's default [`DespawnMode`].
///
/// `bevy_save` will use this when applying snapshots without a specified [`DespawnMode`].
#[derive(Resource, Default, Deref, DerefMut, Clone)]
pub struct AppDespawnMode(Arc<DespawnMode>);

impl AppDespawnMode {
    /// Create a new [`AppDespawnMode`] from the given [`DespawnMode`].
    pub fn new(mode: DespawnMode) -> Self {
        Self(Arc::new(mode))
    }

    /// Override the current [`DespawnMode`].
    pub fn set(&mut self, mode: DespawnMode) {
        self.0 = Arc::new(mode);
    }
}

/// The App's default [`MappingMode`].
///
/// `bevy_save` will use this when applying snapshots without a specified [`MappingMode`].
#[derive(Resource, Default, Deref, DerefMut, Clone)]
pub struct AppMappingMode(Arc<MappingMode>);

impl AppMappingMode {
    /// Create a new [`AppMappingMode`] from the given [`MappingMode`].
    pub fn new(mode: MappingMode) -> Self {
        Self(Arc::new(mode))
    }

    /// Override the current [`MappingMode`].
    pub fn set(&mut self, mode: MappingMode) {
        self.0 = Arc::new(mode);
    }
}

/// [`Applier`] lets you configure how a snapshot will be applied to the [`World`].
pub struct Applier<'a, S> {
    pub(crate) world: &'a mut World,
    pub(crate) snapshot: S,
    pub(crate) map: EntityMap,
    pub(crate) despawn: Option<DespawnMode>,
    pub(crate) mapping: Option<MappingMode>,
    pub(crate) hook: Option<BoxedHook>,
}

impl<'a, S> Applier<'a, S> {
    /// Create a new [`Applier`] with default settings from the world and snapshot.
    pub fn new(world: &'a mut World, snapshot: S) -> Self {
        Self {
            world,
            snapshot,
            map: EntityMap::default(),
            despawn: None,
            mapping: None,
            hook: None,
        }
    }

    /// Map entities to new ids with the [`EntityMap`].
    pub fn map(mut self, map: EntityMap) -> Self {
        self.map = map;
        self
    }

    /// Change how the snapshot affects entities when applying.
    pub fn despawn(mut self, mode: DespawnMode) -> Self {
        self.despawn = Some(mode);
        self
    }

    /// Change how the snapshot maps entities when applying.
    pub fn mapping(mut self, mode: MappingMode) -> Self {
        self.mapping = Some(mode);
        self
    }

    /// Add a [`Hook`] that will run for each entity when applying.
    pub fn hook<F>(mut self, hook: F) -> Self
    where
        F: Hook + 'static,
    {
        self.hook = Some(Box::new(hook));
        self
    }
}
