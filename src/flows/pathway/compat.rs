use bevy::prelude::*;

use crate::prelude::*;

mod sealed {
    use bevy::prelude::*;
    use serde::de::DeserializeSeed;

    use super::PipelineCapture;
    use crate as bevy_save;
    use crate::prelude::*;

    #[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
    pub struct CompatFlow;

    pub struct DeserializeCompat<'a>(<Snapshot as CaptureDeserialize>::Seed<'a>);

    impl<'de> DeserializeSeed<'de> for DeserializeCompat<'_> {
        type Value = PipelineCapture;

        fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Ok(PipelineCapture(self.0.deserialize(deserializer)?))
        }
    }

    impl CaptureSerialize for PipelineCapture {
        type Value<'a>
            = <Snapshot as CaptureSerialize>::Value<'a>
        where
            Self: 'a;

        fn value<'a>(&'a self, world: &'a World) -> Self::Value<'a> {
            <Snapshot as CaptureSerialize>::value(&self.0, world)
        }
    }

    impl CaptureDeserialize for PipelineCapture {
        type Seed<'a> = DeserializeCompat<'a>;

        fn seed(world: &World) -> Self::Seed<'_> {
            DeserializeCompat(<Snapshot as CaptureDeserialize>::seed(world))
        }
    }

    impl<P> CaptureInput<P> for PipelineCapture
    where
        P: Pipeline,
    {
        type Builder = Builder;

        fn builder(_pathway: &P, _world: &World) -> Self::Builder {
            Builder::new()
        }

        fn build(builder: Self::Builder, pathway: &P, world: &World) -> Self {
            PipelineCapture(pathway.capture(BuilderRef::from_parts(world, builder)))
        }
    }

    impl<P> CaptureOutput<P> for PipelineCapture
    where
        P: Pipeline,
    {
        type Builder = Snapshot;
        type Error = Error;

        fn builder(self, _pathway: &P, _world: &mut World) -> Self::Builder {
            self.0
        }

        fn build(
            builder: Self::Builder,
            pathway: &P,
            world: &mut World,
        ) -> Result<Self, Self::Error> {
            pathway.apply(world, &builder)?;
            Ok(PipelineCapture(builder))
        }
    }
}

/// Compatibility wrapper type for using [`Pipeline`] with [`Pathway`].
pub struct PipelineCapture(pub Snapshot);

impl From<Snapshot> for PipelineCapture {
    fn from(value: Snapshot) -> Self {
        Self(value)
    }
}

impl From<PipelineCapture> for Snapshot {
    fn from(value: PipelineCapture) -> Self {
        value.0
    }
}

impl<P> Pathway for P
where
    P: Pipeline + 'static,
{
    type Capture = PipelineCapture;

    type Backend = P::Backend;
    type Format = P::Format;
    type Key<'a> = P::Key<'a>;

    fn key(&self) -> Self::Key<'_> {
        <P as Pipeline>::key(self)
    }

    fn capture(&self, _world: &World) -> impl FlowLabel {
        sealed::CompatFlow
    }

    fn apply(&self, _world: &World) -> impl FlowLabel {
        sealed::CompatFlow
    }
}
