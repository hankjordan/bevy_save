//! Wrappers for remote types

mod boxed;
mod entity;
mod map;

pub use self::{
    boxed::BoxedPartialReflect,
    entity::DynamicEntity,
    map::{
        EntityMap,
        ReflectMap,
    },
};
