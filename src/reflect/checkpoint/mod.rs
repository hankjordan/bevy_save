//! Checkpoint utilities for [`Snapshot`](crate::prelude::Snapshot)s
//! that can be quickly rolled through.

mod ext;
mod state;

use bevy::reflect::FromType;

pub use self::{
    ext::WorldCheckpointExt,
    state::Checkpoints,
};

/// Register this [`TypeData`](bevy::reflect::TypeData) to prevent inclusion in [`Checkpoints`].
#[derive(Clone)]
pub struct ReflectIgnoreCheckpoint;

impl<T> FromType<T> for ReflectIgnoreCheckpoint {
    fn from_type() -> Self {
        Self
    }
}
