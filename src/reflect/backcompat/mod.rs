use thiserror::Error;

pub(crate) mod v0_16;

const VERSION_0_16: semver::Version = semver::Version::new(0, 16, 0);
const VERSION_0_20: semver::Version = semver::Version::new(0, 20, 0);

/// Error thrown if snapshot version is invalid
#[derive(Debug, Error)]
pub enum VersionError {
    /// Unsupported snapshot version
    #[error("Unsupported `bevy_save` snapshot version")]
    Unsupported,

    /// Invalid semver string
    #[error("Invalid semver: `{0}`")]
    Invalid(#[from] semver::Error),
}

/// Snapshot format version
#[derive(Clone, Copy, Default)]
#[non_exhaustive]
pub enum Version {
    /// Snapshot with explicit `rollbacks` field
    V0_16,

    /// Reflect-enabled snapshot with metadata
    #[default]
    V0_20,
}

impl TryFrom<&str> for Version {
    type Error = VersionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let ver: semver::Version = value.parse()?;

        if ver >= VERSION_0_20 {
            Ok(Self::V0_20)
        } else if ver >= VERSION_0_16 {
            Ok(Self::V0_16)
        } else {
            Err(VersionError::Unsupported)
        }
    }
}
