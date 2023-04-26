use std::{
    collections::HashSet,
    marker::PhantomData,
    sync::Arc,
};

use bevy::{
    ecs::{
        entity::EntityMap,
        query::ReadOnlyWorldQuery,
        world::EntityMut,
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

/// A [`Hook`] runs on each [`EntityMut`] when applying a snapshot.
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
/// # let root = world.spawn_empty().id();
/// snapshot.applier(world)
///     .mapping(MappingMode::simple_hooked(move |e| {
///         if e.contains::<Parent>() {
///             e
///         } else {
///             e.set_parent(root)
///         }
///     }))
///     .apply();
/// ```
pub trait Hook: for<'a> Fn(&'a mut EntityMut<'a>) -> &'a mut EntityMut<'a> + Send + Sync {}

impl<T> Hook for T where T: for<'a> Fn(&'a mut EntityMut<'a>) -> &'a mut EntityMut<'a> + Send + Sync {}

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

    /// Same as [`MappingMode::Simple`], but also apply a custom [`Hook`].
    SimpleHooked(BoxedHook),

    /// If unmapped, spawn a new entity.
    ///
    /// `bevy_scene` default
    Strict,

    /// Same as [`MappingMode::Strict`], but also apply a custom [`Hook`].
    StrictHooked(BoxedHook),
}

impl MappingMode {
    /// Create a new instance of [`MappingMode::SimpleHooked`] with the given [`Hook`].
    pub fn simple_hooked<F>(f: F) -> Self
    where
        F: Hook + 'static,
    {
        Self::SimpleHooked(Box::new(f))
    }

    /// Create a new instance of [`MappingMode::StrictHooked`] with the given [`Hook`].
    pub fn strict_hooked<F>(f: F) -> Self
    where
        F: Hook + 'static,
    {
        Self::StrictHooked(Box::new(f))
    }
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
}
