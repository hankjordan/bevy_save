use thiserror::Error;

pub(crate) mod v3;

const VERSION_0: semver::Version = semver::Version::new(0, 2, 0);
const VERSION_1: semver::Version = semver::Version::new(0, 6, 0);
const VERSION_2: semver::Version = semver::Version::new(0, 15, 0);
const VERSION_3: semver::Version = semver::Version::new(0, 16, 0);
const VERSION_4: semver::Version = semver::Version::new(1, 0, 0);

/// Error thrown if snapshot version is invalid
#[derive(Debug, Error)]
pub enum VersionError {
    /// Unsupported version
    #[error("Unsupported `bevy_save` version")]
    Unsupported,

    /// Invalid semver string
    #[error("Invalid semver: `{0}`")]
    Invalid(#[from] semver::Error),
}

/// Snapshot format version
#[derive(Clone, Copy, Default)]
#[non_exhaustive]
pub enum SnapshotVersion {
    /// Snapshot with explicit `rollbacks` field, dynamically cloned values,
    /// index-only entities, and nested `entities` map
    ///
    /// Not currently supported
    V0,

    /// Snapshot with explicit `rollbacks` field, dynamically cloned values, and
    /// index-only entities
    ///
    /// Not currently supported
    V1,

    /// Snapshot with explicit `rollbacks` field and dynamically cloned values
    ///
    /// Not currently supported
    V2,

    /// Snapshot with explicit `rollbacks` field
    V3,

    /// Reflect-enabled snapshot with versioning
    #[default]
    V4,
}

impl TryFrom<&str> for SnapshotVersion {
    type Error = VersionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let ver: semver::Version = value.parse()?;

        if ver >= VERSION_4 {
            Ok(Self::V4)
        } else if ver >= VERSION_3 {
            Ok(Self::V3)
        } else if ver >= VERSION_2 {
            Ok(Self::V2)
        } else if ver >= VERSION_1 {
            Ok(Self::V1)
        } else if ver >= VERSION_0 {
            Ok(Self::V0)
        } else {
            Err(VersionError::Unsupported)
        }
    }
}
