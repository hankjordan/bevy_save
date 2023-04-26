use std::{
    collections::HashSet,
    marker::PhantomData,
    sync::Arc,
};

use bevy::{
    ecs::{
        entity::EntityMap,
        query::ReadOnlyWorldQuery,
        reflect::ReflectMapEntities,
        world::EntityMut,
    },
    prelude::*,
    reflect::TypeRegistration,
};

use crate::{
    entity::SaveableEntity,
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

/// A [`Mapper`] runs on each [`EntityMut`] when applying a snapshot.
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
///     .mapping(MappingMode::simple_map(move |e| {
///         if e.contains::<Parent>() {
///             e
///         } else {
///             e.set_parent(root)
///         }
///     }))
///     .apply();
/// ```
pub trait Mapper: for<'a> Fn(&'a mut EntityMut<'a>) -> &'a mut EntityMut<'a> + Send + Sync {}

impl<T> Mapper for T where
    T: for<'a> Fn(&'a mut EntityMut<'a>) -> &'a mut EntityMut<'a> + Send + Sync
{
}

/// A boxed [`Mapper`].
pub type BoxedMapper = Box<dyn Mapper>;

/// Determines how the snapshot will map entities when applied.
#[derive(Default)]
pub enum MappingMode {
    /// If unmapped, attempt a one-to-one mapping. If that fails, spawn a new entity.
    ///
    /// `bevy_save` default
    #[default]
    Simple,

    /// Same as [`MappingMode::Simple`], but also apply a custom mapping function.
    SimpleMap(BoxedMapper),

    /// If unmapped, spawn a new entity.
    ///
    /// `bevy_scene` default
    Strict,

    /// Same as [`MappingMode::Strict`], but also apply a custom mapping function.
    StrictMap(BoxedMapper),
}

impl MappingMode {
    /// Create a new instance of [`MappingMode::SimpleMap`] with the given [`Mapper`].
    pub fn simple_map<F>(f: F) -> Self
    where
        F: Mapper + 'static,
    {
        Self::SimpleMap(Box::new(f))
    }

    /// Create a new instance of [`MappingMode::StrictMap`] with the given [`Mapper`].
    pub fn strict_map<F>(f: F) -> Self
    where
        F: Mapper + 'static,
    {
        Self::StrictMap(Box::new(f))
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
    world: &'a mut World,
    snapshot: S,
    map: EntityMap,
    despawn: Option<DespawnMode>,
    mapping: Option<MappingMode>,
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

pub(crate) struct RawSnapshot {
    pub(crate) resources: Vec<Box<dyn Reflect>>,
    pub(crate) entities: Vec<SaveableEntity>,
}

impl RawSnapshot {
    fn from_world_with_filter<F>(world: &World, filter: F) -> Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        let registry_arc = world.resource::<AppTypeRegistry>();
        let registry = registry_arc.read();

        let saveables = world.resource::<SaveableRegistry>();

        // Resources

        let resources = saveables
            .types()
            .filter_map(|name| registry.get_with_name(name))
            .filter(&filter)
            .filter_map(|reg| reg.data::<ReflectResource>())
            .filter_map(|res| res.reflect(world))
            .map(|reflect| reflect.clone_value())
            .collect::<Vec<_>>();

        // Entities

        let mut entities = Vec::new();

        for entity in world.iter_entities().map(|entity| entity.id()) {
            let mut entry = SaveableEntity {
                entity: entity.index(),
                components: Vec::new(),
            };

            let entity = world.entity(entity);

            for component_id in entity.archetype().components() {
                let reflect = world
                    .components()
                    .get_info(component_id)
                    .filter(|info| saveables.contains(info.name()))
                    .and_then(|info| info.type_id())
                    .and_then(|id| registry.get(id))
                    .filter(&filter)
                    .and_then(|reg| reg.data::<ReflectComponent>())
                    .and_then(|reflect| reflect.reflect(entity));

                if let Some(reflect) = reflect {
                    entry.components.push(reflect.clone_value());
                }
            }

            entities.push(entry);
        }

        Self {
            resources,
            entities,
        }
    }
}

impl<'a> Applier<'a, &'a RawSnapshot> {
    fn apply(self) -> Result<(), SaveableError> {
        let registry_arc = self.world.resource::<AppTypeRegistry>().clone();
        let registry = registry_arc.read();

        // Resources

        for resource in &self.snapshot.resources {
            let reg = registry
                .get_with_name(resource.type_name())
                .ok_or_else(|| SaveableError::UnregisteredType {
                    type_name: resource.type_name().to_string(),
                })?;

            let data = reg.data::<ReflectResource>().ok_or_else(|| {
                SaveableError::UnregisteredResource {
                    type_name: resource.type_name().to_string(),
                }
            })?;

            data.insert(self.world, resource.as_reflect());

            if let Some(mapper) = reg.data::<ReflectMapEntities>() {
                mapper
                    .map_entities(self.world, &self.map)
                    .map_err(SaveableError::MapEntitiesError)?;
            }
        }

        // Entities

        let despawn_default = self
            .world
            .get_resource::<AppDespawnMode>()
            .cloned()
            .unwrap_or_default();

        let despawn = self.despawn.as_ref().unwrap_or(&despawn_default);

        match despawn {
            DespawnMode::Missing | DespawnMode::MissingWith(_) => {
                let valid = self
                    .snapshot
                    .entities
                    .iter()
                    .map(|e| e.try_map(&self.map))
                    .collect::<HashSet<_>>();

                let mut invalid = self
                    .world
                    .iter_entities()
                    .map(|e| e.id())
                    .filter(|e| !valid.contains(e))
                    .collect::<Vec<_>>();

                if let DespawnMode::MissingWith(filter) = despawn {
                    let matches = filter.collect(self.world);
                    invalid.retain(|e| matches.contains(e));
                }

                for entity in invalid {
                    self.world.despawn(entity);
                }
            }

            DespawnMode::Unmapped | DespawnMode::UnmappedWith(_) => {
                let valid = self
                    .snapshot
                    .entities
                    .iter()
                    .filter_map(|e| e.map(&self.map))
                    .collect::<HashSet<_>>();

                let mut invalid = self
                    .world
                    .iter_entities()
                    .map(|e| e.id())
                    .filter(|e| !valid.contains(e))
                    .collect::<Vec<_>>();

                if let DespawnMode::UnmappedWith(filter) = despawn {
                    let matches = filter.collect(self.world);
                    invalid.retain(|e| matches.contains(e));
                }

                for entity in invalid {
                    self.world.despawn(entity);
                }
            }
            DespawnMode::All => {
                let invalid = self
                    .world
                    .iter_entities()
                    .map(|e| e.id())
                    .collect::<Vec<_>>();

                for entity in invalid {
                    self.world.despawn(entity);
                }
            }
            DespawnMode::AllWith(filter) => {
                let invalid = filter.collect(self.world);

                for entity in invalid {
                    self.world.despawn(entity);
                }
            }
            DespawnMode::None => {}
        }

        let mapping_default = self
            .world
            .get_resource::<AppMappingMode>()
            .cloned()
            .unwrap_or_default();

        let mapping = self.mapping.as_ref().unwrap_or(&mapping_default);

        let fallback = if let MappingMode::Simple = &mapping {
            let mut fallback = EntityMap::default();

            for entity in self.world.iter_entities() {
                fallback.insert(Entity::from_raw(entity.id().index()), entity.id());
            }

            fallback
        } else {
            EntityMap::default()
        };

        // Apply snapshot entities
        for saved in &self.snapshot.entities {
            let index = saved.entity;

            let entity = saved
                .map(&self.map)
                .or_else(|| fallback.get(Entity::from_raw(index)).ok())
                .unwrap_or_else(|| self.world.spawn_empty().id());

            let entity_mut = &mut self.world.entity_mut(entity);

            for component in &saved.components {
                let reg = registry
                    .get_with_name(component.type_name())
                    .ok_or_else(|| SaveableError::UnregisteredType {
                        type_name: component.type_name().to_string(),
                    })?;

                let data = reg.data::<ReflectComponent>().ok_or_else(|| {
                    SaveableError::UnregisteredComponent {
                        type_name: component.type_name().to_string(),
                    }
                })?;

                data.apply_or_insert(entity_mut, &**component);
            }

            if let MappingMode::SimpleMap(mapper) | MappingMode::StrictMap(mapper) = &mapping {
                mapper(entity_mut);
            }
        }

        for reg in registry.iter() {
            if let Some(mapper) = reg.data::<ReflectMapEntities>() {
                mapper
                    .map_entities(self.world, &self.map)
                    .map_err(SaveableError::MapEntitiesError)?;
            }
        }

        Ok(())
    }
}

impl CloneReflect for RawSnapshot {
    fn clone_value(&self) -> Self {
        Self {
            resources: self.resources.clone_value(),
            entities: self.entities.iter().map(|e| e.clone_value()).collect(),
        }
    }
}

/// A rollback snapshot of the game state.
///
/// [`Rollback`] excludes types that opt out of rollback.
pub struct Rollback {
    pub(crate) snapshot: RawSnapshot,
}

impl Rollback {
    /// Returns a [`Rollback`] of the current [`World`] state.
    ///
    /// This excludes [`Rollbacks`] and any saveable that ignores rollbacking.
    pub fn from_world(world: &World) -> Self {
        Self::from_world_with_filter(world, |_| true)
    }

    /// Returns a [`Rollback`] of the current [`World`] state, filtered by `filter`.
    ///
    /// This excludes [`Rollbacks`] and any saveable that ignores rollbacking.
    pub fn from_world_with_filter<F>(world: &World, filter: F) -> Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        let registry = world.resource::<SaveableRegistry>();

        let snapshot = RawSnapshot::from_world_with_filter(world, |reg| {
            registry.can_rollback(reg.type_name()) && filter(reg)
        });

        Self { snapshot }
    }

    /// Apply the [`Rollback`] to the [`World`].
    ///
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply(&self, world: &mut World) -> Result<(), SaveableError> {
        self.applier(world).apply()
    }

    /// Create an [`Applier`] from the [`Rollback`] and the [`World`].
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy::ecs::entity::EntityMap;
    /// # use bevy_save::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = &mut app.world;
    /// let rollback = Rollback::from_world(world);
    ///
    /// rollback
    ///     .applier(world)
    ///     .map(EntityMap::default())
    ///     .despawn(DespawnMode::default())
    ///     .mapping(MappingMode::default())
    ///     .apply();
    /// ```
    pub fn applier<'a>(&'a self, world: &'a mut World) -> Applier<'a, &'a Self> {
        Applier::new(world, self)
    }

    /// Create an owning [`Applier`] from the [`Rollback`] and the [`World`].
    pub fn into_applier(self, world: &mut World) -> Applier<Self> {
        Applier::new(world, self)
    }
}

macro_rules! impl_rollback_applier {
    ($t:ty) => {
        impl<'a> Applier<'a, $t> {
            /// Apply the [`Rollback`].
            ///
            /// # Errors
            /// - See [`SaveableError`]
            pub fn apply(self) -> Result<(), SaveableError> {
                let applier = Applier {
                    world: self.world,
                    snapshot: &self.snapshot.snapshot,
                    map: self.map,
                    despawn: self.despawn,
                    mapping: self.mapping,
                };

                applier.apply()
            }
        }
    };
}

impl_rollback_applier!(Rollback);
impl_rollback_applier!(&'a Rollback);

impl CloneReflect for Rollback {
    fn clone_value(&self) -> Self {
        Self {
            snapshot: self.snapshot.clone_value(),
        }
    }
}

/// A complete snapshot of the game state.
///
/// Can be serialized via [`SnapshotSerializer`] and deserialized via [`SnapshotDeserializer`].
pub struct Snapshot {
    pub(crate) snapshot: RawSnapshot,
    pub(crate) rollbacks: Rollbacks,
}

impl Snapshot {
    /// Returns a [`Snapshot`] of the current [`World`] state.
    /// Includes [`Rollbacks`].
    pub fn from_world(world: &World) -> Self {
        Self::from_world_with_filter(world, |_| true)
    }

    /// Returns a [`Snapshot`] of the current [`World`] state filtered by `filter`.
    pub fn from_world_with_filter<F>(world: &World, filter: F) -> Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        let snapshot = RawSnapshot::from_world_with_filter(world, filter);
        let rollbacks = world.resource::<Rollbacks>().clone_value();

        Self {
            snapshot,
            rollbacks,
        }
    }

    /// Apply the [`Snapshot`] to the [`World`], restoring it to the saved state.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply(&self, world: &mut World) -> Result<(), SaveableError> {
        self.applier(world).apply()
    }

    /// Create an [`Applier`] from the [`Snapshot`] and the [`World`].
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy::ecs::entity::EntityMap;
    /// # use bevy_save::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = &mut app.world;
    /// let snapshot = Snapshot::from_world(world);
    ///
    /// snapshot
    ///     .applier(world)
    ///     .map(EntityMap::default())
    ///     .despawn(DespawnMode::default())
    ///     .mapping(MappingMode::default())
    ///     .apply();
    /// ```
    pub fn applier<'a>(&'a self, world: &'a mut World) -> Applier<&Self> {
        Applier::new(world, self)
    }

    /// Create an owning [`Applier`] from the [`Snapshot`] and the [`World`].
    pub fn into_applier(self, world: &mut World) -> Applier<Self> {
        Applier::new(world, self)
    }
}

macro_rules! impl_snapshot_applier {
    ($t:ty) => {
        impl<'a> Applier<'a, $t> {
            /// Apply the [`Snapshot`].
            ///
            /// # Errors
            /// - See [`SaveableError`]
            pub fn apply(self) -> Result<(), SaveableError> {
                let applier = Applier {
                    world: self.world,
                    snapshot: &self.snapshot.snapshot,
                    map: self.map,
                    despawn: self.despawn,
                    mapping: self.mapping,
                };

                applier.apply()?;

                self.world
                    .insert_resource(self.snapshot.rollbacks.clone_value());

                Ok(())
            }
        }
    };
}

impl_snapshot_applier!(Snapshot);
impl_snapshot_applier!(&'a Snapshot);

impl CloneReflect for Snapshot {
    fn clone_value(&self) -> Self {
        Self {
            snapshot: self.snapshot.clone_value(),
            rollbacks: self.rollbacks.clone_value(),
        }
    }
}
