//! Wrappers for remote types

mod entity;
mod map;
mod value;

pub use self::{
    entity::DynamicEntity,
    map::{
        EntityMap,
        ReflectMap,
    },
    value::DynamicValue,
};
