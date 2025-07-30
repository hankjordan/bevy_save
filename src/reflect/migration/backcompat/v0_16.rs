use bevy::reflect::Reflect;

use crate::reflect::{
    EntityMap,
    ReflectMap,
};

#[derive(Reflect)]
pub(crate) struct SnapshotV0_16 {
    pub(crate) entities: EntityMap,
    pub(crate) resources: ReflectMap,
    #[cfg(feature = "checkpoints")]
    pub(crate) rollbacks: Option<CheckpointsV0_16>,
}

#[derive(Reflect)]
#[cfg(feature = "checkpoints")]
pub(crate) struct CheckpointV0_16 {
    pub(crate) entities: EntityMap,
    pub(crate) resources: ReflectMap,
}

#[derive(Reflect)]
#[cfg(feature = "checkpoints")]
pub(crate) struct CheckpointsV0_16 {
    pub(crate) checkpoints: Vec<CheckpointV0_16>,
    pub(crate) active: Option<usize>,
}
