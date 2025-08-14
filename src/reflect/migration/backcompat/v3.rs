use bevy::reflect::Reflect;

use crate::reflect::{
    EntityMap,
    ReflectMap,
};

#[derive(Reflect)]
pub(crate) struct SnapshotV3 {
    pub(crate) entities: EntityMap,
    pub(crate) resources: ReflectMap,
    #[cfg(feature = "checkpoints")]
    pub(crate) rollbacks: Option<CheckpointsV3>,
}

#[derive(Reflect)]
#[cfg(feature = "checkpoints")]
pub(crate) struct CheckpointV3 {
    pub(crate) entities: EntityMap,
    pub(crate) resources: ReflectMap,
}

#[derive(Reflect)]
#[cfg(feature = "checkpoints")]
pub(crate) struct CheckpointsV3 {
    pub(crate) checkpoints: Vec<CheckpointV3>,
    pub(crate) active: Option<usize>,
}
