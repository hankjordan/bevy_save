use std::marker::PhantomData;

use bevy::prelude::*;
use serde::{
    Deserialize,
    Serialize,
    de::DeserializeSeed,
};

/// Implemented for capture types which can be used during capture flows
pub trait CaptureInput<P> {
    /// The builder type passed to flow systems
    type Builder: 'static;

    /// Creates the builder from the given [`Pathway`](crate::prelude::Pathway) and [`World`]
    fn builder(pathway: &P, world: &World) -> Self::Builder;

    /// Builds the capture type
    fn build(builder: Self::Builder, pathway: &P, world: &World) -> Self;
}

/// Implemented for capture types which can be used during apply flows
pub trait CaptureOutput<P>: Sized {
    /// The builder type passed to flow systems
    type Builder: 'static;

    /// Error potentially thrown while building
    type Error;

    /// Creates the builder from the given [`Pathway`](crate::prelude::Pathway) and [`World`]
    fn builder(self, pathway: &P, world: &mut World) -> Self::Builder;

    /// Builds the capture type
    ///
    /// # Errors
    /// - If the building process fails
    fn build(builder: Self::Builder, pathway: &P, world: &mut World) -> Result<Self, Self::Error>;
}

/// Implemented for capture types capable of serialization
///
/// Useful for when these types need access to the [`AppTypeRegistry`] or other [`World`] state
pub trait CaptureSerialize {
    /// The serializable value
    type Value<'a>: Serialize + Send + Sync
    where
        Self: 'a;

    /// Convert a reference into a serializable value
    fn value<'a>(&'a self, world: &'a World) -> Self::Value<'a>;
}

/// Implemented for capture types capable of deserialization
///
/// Useful for when these types need access to the [`AppTypeRegistry`] or other [`World`] state
pub trait CaptureDeserialize {
    /// The seed used for deserialization
    type Seed<'a>: for<'de> DeserializeSeed<'de, Value = Self> + Send + Sync;

    /// Create a deserialization seed from the world
    fn seed(world: &World) -> Self::Seed<'_>;
}

impl<T, P> CaptureInput<P> for T
where
    T: Default + 'static,
{
    type Builder = T;

    fn builder(_pathway: &P, _world: &World) -> Self::Builder {
        T::default()
    }

    fn build(builder: Self::Builder, _pathway: &P, _world: &World) -> Self {
        builder
    }
}

impl<T, P> CaptureOutput<P> for T
where
    T: Default + 'static,
{
    type Builder = T;
    type Error = ();

    fn builder(self, _pathway: &P, _world: &mut World) -> Self::Builder {
        self
    }

    fn build(
        builder: Self::Builder,
        _pathway: &P,
        _world: &mut World,
    ) -> Result<Self, Self::Error> {
        Ok(builder)
    }
}

impl<T> CaptureSerialize for T
where
    T: Serialize + Send + Sync,
{
    type Value<'a>
        = &'a T
    where
        T: 'a;

    fn value<'a>(&'a self, _world: &'a World) -> Self::Value<'a> {
        self
    }
}

impl<T> CaptureDeserialize for T
where
    T: for<'de> Deserialize<'de> + Send + Sync,
{
    type Seed<'a> = PhantomData<T>;

    fn seed(_world: &World) -> Self::Seed<'_> {
        PhantomData
    }
}

#[cfg(feature = "reflect")]
mod reflect {
    use bevy::prelude::*;

    use crate::{
        prelude::*,
        reflect::{
            SnapshotDeserializerArc,
            SnapshotSerializerArc,
        },
    };

    impl<P> CaptureInput<P> for Snapshot {
        type Builder = Builder;

        fn builder(_pathway: &P, _world: &World) -> Self::Builder {
            Builder::new()
        }

        fn build(builder: Self::Builder, _pathway: &P, _world: &World) -> Self {
            builder.build()
        }
    }

    impl<P> CaptureOutput<P> for Snapshot {
        type Builder = Applier<'static>;
        type Error = Error;

        fn builder(self, _pathway: &P, _world: &mut World) -> Self::Builder {
            Applier::new(self)
        }

        fn build(
            builder: Self::Builder,
            _pathway: &P,
            world: &mut World,
        ) -> Result<Self, Self::Error> {
            let mut applier = ApplierRef::from_parts(world, builder);

            applier.apply()?;

            Ok(applier
                .into_inner()
                .snapshot
                .try_into_owned()
                .expect("Invalid input"))
        }
    }

    impl CaptureSerialize for Snapshot {
        type Value<'a>
            = SnapshotSerializerArc<'a>
        where
            Self: 'a;

        fn value<'a>(&'a self, world: &'a World) -> Self::Value<'a> {
            SnapshotSerializerArc::new(self, world.resource::<AppTypeRegistry>().clone().0)
        }
    }

    impl CaptureDeserialize for Snapshot {
        type Seed<'a> = SnapshotDeserializerArc;

        fn seed(world: &World) -> Self::Seed<'_> {
            SnapshotDeserializerArc::new(world.resource::<AppTypeRegistry>().clone().0)
        }
    }
}
