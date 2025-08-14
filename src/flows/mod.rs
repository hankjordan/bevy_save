//! [`Flows`] are chains of systems used to modularize the capture and apply
//! process.

use bevy::ecs::{
    define_label,
    intern::Interned,
    system::{
        self,
        System,
    },
};
use thiserror::Error;

mod ext;
pub mod pathway;
mod registry;
mod systems;

pub use self::{
    ext::AppFlowExt,
    registry::Flows,
    systems::{
        Flow,
        IntoFlowSystems,
    },
};

/// An error that may occur while running a flow.
#[derive(Error, Debug)]
pub enum FlowError {
    /// No [`AppTypeRegistry`](bevy::prelude::AppTypeRegistry) found
    #[error("could not find `AppTypeRegistry`")]
    NoTypeRegistry,

    /// No [`Flows`] found for input type
    #[error("No flows with input `{0:?}` found")]
    NotRegistered(&'static str),

    /// [`Flow`] has not been registered with [`Flows`]
    #[error("the flow `{0:?}` was not registered")]
    NotFound(InternedFlowLabel),
}

/// Type alias for boxed flow systems that have mutable access to
/// [`World`](bevy::ecs::world::World)
pub type FlowSystem<In> = Box<dyn System<In = system::In<In>, Out = In>>;

define_label!(
    /// A strongly-typed class of labels used to identify a [`Flow`].
    #[diagnostic::on_unimplemented(
        note = "consider annotating `{Self}` with `#[derive(FlowLabel)]`"
    )]
    FlowLabel,
    FLOW_LABEL_INTERNER
);

/// A shorthand for `Interned<dyn FlowLabel>`.
pub type InternedFlowLabel = Interned<dyn FlowLabel>;
