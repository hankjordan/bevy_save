use std::str::FromStr;

/// Conversion trait for anything that can be used as a [`Version`](semver::Version).
pub trait IntoVersion {
    /// Converts the type into [`Version`](semver::Version).
    ///
    /// # Errors
    /// - If the type does not represent a valid [`Version`](semver::Version).
    fn into_version(self) -> Result<semver::Version, semver::Error>;
}

impl IntoVersion for &str {
    fn into_version(self) -> Result<semver::Version, semver::Error> {
        semver::Version::from_str(self)
    }
}

impl IntoVersion for String {
    fn into_version(self) -> Result<semver::Version, semver::Error> {
        semver::Version::from_str(self.as_ref())
    }
}

impl IntoVersion for semver::Version {
    fn into_version(self) -> Result<semver::Version, semver::Error> {
        Ok(self)
    }
}

/// Borrowed or owned value
#[derive(Debug)]
pub enum MaybeRef<'a, T> {
    /// Owned value
    Owned(T),
    /// Reference to value
    Borrowed(&'a T),
}

impl<T> MaybeRef<'static, T> {
    /// Converts the [`MaybeRef`] into an owned value.
    ///
    /// # Errors
    /// - If the value is not owned
    pub fn try_into_owned(self) -> Result<T, Self> {
        if let Self::Owned(value) = self {
            Ok(value)
        } else {
            Err(self)
        }
    }
}

impl<T> std::ops::Deref for MaybeRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(value) => value,
            Self::Borrowed(value) => value,
        }
    }
}

impl<T> From<T> for MaybeRef<'static, T> {
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<'a, T> From<&'a T> for MaybeRef<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::Borrowed(value)
    }
}

/// Mutably borrowed or owned value
pub enum MaybeMut<'a, T> {
    /// Owned value
    Owned(T),
    /// Mutable reference to value
    Mut(&'a mut T),
}

impl<T> std::ops::Deref for MaybeMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(value) => value,
            Self::Mut(value) => value,
        }
    }
}

impl<T> std::ops::DerefMut for MaybeMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Owned(value) => value,
            Self::Mut(value) => value,
        }
    }
}

impl<T> Default for MaybeMut<'_, T>
where
    T: Default,
{
    fn default() -> Self {
        Self::Owned(T::default())
    }
}

impl<T> From<T> for MaybeMut<'static, T> {
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<'a, T> From<&'a mut T> for MaybeMut<'a, T> {
    fn from(value: &'a mut T) -> Self {
        Self::Mut(value)
    }
}
