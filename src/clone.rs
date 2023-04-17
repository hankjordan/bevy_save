use bevy::prelude::*;

/// Clone-like trait for duplicating [`Reflect`]-related types.
pub trait CloneReflect {
    /// Clone the value via Reflection.
    #[must_use]
    fn clone_value(&self) -> Self;
}

impl CloneReflect for Vec<Box<dyn Reflect>> {
    fn clone_value(&self) -> Self {
        let mut result = Vec::new();

        for reflect in self {
            result.push(reflect.clone_value());
        }

        result
    }
}