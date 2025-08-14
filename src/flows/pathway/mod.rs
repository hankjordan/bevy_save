//! [`Pathway`] connects all of the pieces together, defining how your
//! application state is captured, applied, saved, and loaded.
//!
//! Unlike [`Pipeline`], [`Pathway`] allows you to use [`Flow`]s and custom
//! capture types.

use bevy::prelude::*;

use crate::prelude::*;

mod capture;
mod ext;

pub use self::{
    capture::{
        CaptureDeserialize,
        CaptureInput,
        CaptureOutput,
        CaptureSerialize,
    },
    ext::{
        AppPathwayExt,
        WorldPathwayExt,
    },
};

#[cfg(feature = "reflect")]
mod compat;

#[cfg(feature = "reflect")]
pub use self::compat::PipelineCapture;

/// Trait that defines how exactly your app saves and loads.
///
/// Unlike [`Pipeline`], [`Pathway`] allows you to define custom capture types
/// and utilize [`Flow`]s.
pub trait Pathway {
    /// The type to be captured from and applied to the world
    type Capture: 'static;

    /// The interface between the saver / loader and data storage.
    type Backend: for<'a> Backend<Self::Key<'a>> + Send + Sync + 'static;
    /// The format used for serializing and deserializing data.
    type Format: Format;

    /// Used to uniquely identify each saved capture.
    type Key<'a>;

    /// Retrieve the unique identifier for the capture being processed by the
    /// [`Pathway`].
    fn key(&self) -> Self::Key<'_>;

    /// The label of the capture [`Flow`] the [`Pathway`] will use
    fn capture(&self, world: &World) -> impl FlowLabel;

    /// The label of the apply [`Flow`] the [`Pathway`] will use
    fn apply(&self, world: &World) -> impl FlowLabel;
}

#[cfg(test)]
mod test {
    use serde::{
        Deserialize,
        Serialize,
    };

    use super::*;
    use crate as bevy_save;

    struct ExamplePathway {
        pos: (i32, i32, i32),
    }

    #[derive(Serialize, Deserialize)]
    struct ExamplePathwayCapture {
        pos: (i32, i32, i32),
        transforms: Vec<(Entity, Transform)>,
    }

    #[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
    struct ExamplePathwayCaptureFlow;

    #[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, FlowLabel)]
    struct ExamplePathwayApplyFlow;

    impl Pathway for ExamplePathway {
        type Capture = ExamplePathwayCapture;

        type Backend = DefaultDebugBackend;
        type Format = DefaultDebugFormat;
        type Key<'a> = String;

        fn key(&self) -> Self::Key<'_> {
            let (x, y, z) = self.pos;
            format!("sav-{x}-{y}-{z}")
        }

        fn capture(&self, _world: &World) -> impl FlowLabel {
            ExamplePathwayCaptureFlow
        }

        fn apply(&self, _world: &World) -> impl FlowLabel {
            ExamplePathwayApplyFlow
        }
    }
}
