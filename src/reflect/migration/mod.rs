//! Migrations: versioned snapshots

use std::{
    collections::{
        HashMap,
        HashSet,
    },
    marker::PhantomData,
    sync::OnceLock,
};

use bevy::reflect::{
    FromReflect,
    FromType,
    GetTypeRegistration,
    PartialReflect,
    Reflect,
    ReflectFromReflect,
    TypePath,
    TypeRegistration,
};
use semver::Version;

pub(crate) mod backcompat;

pub use backcompat::{
    SnapshotVersion,
    VersionError,
};

use crate::IntoVersion;

type TransformFn = Box<dyn Fn(&dyn PartialReflect) -> Option<Box<dyn Reflect>> + Send + Sync>;

struct MigrationStep {
    registration: TypeRegistration,
    from_reflect: ReflectFromReflect,
    transform: TransformFn,
}

struct MigratorData {
    type_paths: HashSet<&'static str>,
    steps: HashMap<Version, MigrationStep>,
}

impl std::fmt::Debug for MigratorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigratorData")
            .field("type_paths", &self.type_paths)
            .field("steps", &self.steps.keys())
            .finish()
    }
}

impl MigratorData {
    fn new() -> Self {
        Self {
            type_paths: HashSet::new(),
            steps: HashMap::new(),
        }
    }
}

/// Defines a migration upgrade flow for a type.
pub struct Migrator<In = ()> {
    data: MigratorData,
    _marker: PhantomData<In>,
}

impl Migrator {
    /// Creates a default [`Migrator`] for the given first step output.
    pub fn new<Out>(version: impl IntoVersion) -> Migrator<Out>
    where
        Out: FromReflect + TypePath + GetTypeRegistration,
    {
        Migrator {
            data: MigratorData::new(),
            _marker: PhantomData::<()>,
        }
        .add_step::<Out>(version, None)
    }
}

impl<In> Migrator<In> {
    fn add_step<Out>(
        mut self,
        version: impl IntoVersion,
        transform: Option<TransformFn>,
    ) -> Migrator<Out>
    where
        Out: FromReflect + TypePath + GetTypeRegistration,
    {
        self.data.type_paths.insert(Out::type_path());

        self.data.steps.insert(
            version.into_version().expect("Invalid version string"),
            MigrationStep {
                registration: Out::get_type_registration(),
                from_reflect: FromType::<Out>::from_type(),
                transform: transform.unwrap_or_else(|| {
                    Box::new(move |r| Out::from_reflect(r).map(|r| Box::new(r).into_reflect()))
                }),
            },
        );

        Migrator {
            data: self.data,
            _marker: PhantomData,
        }
    }

    /// Defines a migration step with the given version and transformation function.
    pub fn version<Out>(
        self,
        version: impl IntoVersion,
        step: impl Fn(In) -> Option<Out> + Send + Sync + 'static,
    ) -> Migrator<Out>
    where
        In: FromReflect + TypePath + GetTypeRegistration,
        Out: FromReflect + TypePath + GetTypeRegistration,
    {
        self.add_step(
            version,
            Some(Box::new(move |r| {
                In::from_reflect(r)
                    .and_then(&step)
                    .map(|r| Box::new(r).into_reflect())
            })),
        )
    }
}

/// [`Migrate`] allows reflect-enabled types to define a [`Migrator`] which can
/// transform older versions of the type into the current version.
pub trait Migrate: TypePath + Sized {
    /// Returns the [`Migrator`] for the type.
    fn migrator() -> Migrator<Self>;
}

/// Type data that represents the [`Migrate`] trait and allows it to be used dynamically.
///
/// [`Migrate`] allows reflect-enabled types to define a [`Migrator`] which can
/// transform older versions of the type into the current version.
#[derive(Clone)]
pub struct ReflectMigrate {
    migrate: fn(&dyn PartialReflect, Version) -> Option<Box<dyn Reflect>>,
    matches: fn(&str) -> bool,
    registration: fn(Version) -> Option<&'static TypeRegistration>,
    version: fn() -> Option<&'static Version>,
}

impl ReflectMigrate {
    /// Upgrades the versioned [`PartialReflect`] value with the reflected [`Migrator`].
    pub fn migrate(
        &self,
        value: &dyn PartialReflect,
        version: impl IntoVersion,
    ) -> Option<Box<dyn Reflect>> {
        (self.migrate)(value, version.into_version().ok()?)
    }

    /// Returns `true` if the [`Migrator`] can migrate the given type path.
    pub fn matches(&self, type_path: &str) -> bool {
        (self.matches)(type_path)
    }

    /// Returns the stored [`TypeRegistration`] for the given version.
    pub fn registration(&self, version: impl IntoVersion) -> Option<&TypeRegistration> {
        (self.registration)(version.into_version().ok()?)
    }

    /// Returns the latest registered version for the type.
    pub fn version(&self) -> Option<&Version> {
        (self.version)()
    }
}

impl<T: Migrate> FromType<T> for ReflectMigrate {
    fn from_type() -> Self {
        static CELL: OnceLock<MigratorData> = OnceLock::new();

        ReflectMigrate {
            migrate: |value, version| {
                let data = CELL.get_or_init(|| T::migrator().data);

                // Order steps by version
                let mut steps = data
                    .steps
                    .iter()
                    .filter(|(v, _)| v >= &&version)
                    .collect::<Vec<_>>();

                steps.sort_by_key(|(v, _)| *v);

                let mut it = steps.into_iter();

                let value = it
                    .next()
                    .and_then(|(_, s)| s.from_reflect.from_reflect(value))?;

                it.try_fold(value, |acc, (_, step)| (step.transform)(&*acc))
            },
            matches: |type_path| {
                let data = CELL.get_or_init(|| T::migrator().data);
                data.type_paths.contains(type_path)
            },
            registration: |version| {
                let data = CELL.get_or_init(|| T::migrator().data);

                data.steps
                    .iter()
                    .find(|(v, _)| v == &&version)
                    .map(|(_, s)| &s.registration)
            },
            version: || {
                let data = CELL.get_or_init(|| T::migrator().data);
                data.steps.keys().max()
            },
        }
    }
}
