use bevy::reflect::TypeRegistry;

/// Clone-like trait for duplicating [`Reflect`](bevy::reflect::Reflect) types.
///
/// Any type that does not implement [`FromReflect`](bevy::reflect::FromReflect) will be converted into a Dynamic type.
pub trait CloneReflect {
    /// Clone the value using Reflection.
    #[must_use]
    fn clone_reflect(&self, registry: &TypeRegistry) -> Self;
}
