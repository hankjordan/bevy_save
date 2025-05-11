use bevy::{
    reflect::{
        PartialReflect,
        ReflectFromReflect,
        TypeRegistry,
    },
    scene::DynamicEntity,
};

/// Clone-like trait for duplicating [`Reflect`](bevy::reflect::Reflect) types.
///
/// Any type that does not implement [`FromReflect`](bevy::reflect::FromReflect) will be converted into a Dynamic type.
pub trait CloneReflect {
    /// Clone the value using reflection.
    #[must_use]
    fn clone_reflect(&self, registry: &TypeRegistry) -> Self;
}

impl<T> CloneReflect for Option<T>
where
    T: CloneReflect,
{
    fn clone_reflect(&self, registry: &TypeRegistry) -> Self {
        self.as_ref().map(|t| t.clone_reflect(registry))
    }
}

impl<T> CloneReflect for Vec<T>
where
    T: CloneReflect,
{
    fn clone_reflect(&self, registry: &TypeRegistry) -> Self {
        self.iter().map(|t| t.clone_reflect(registry)).collect()
    }
}

impl CloneReflect for Box<dyn PartialReflect> {
    fn clone_reflect(&self, registry: &TypeRegistry) -> Self {
        registry
            .get(self.get_represented_type_info().unwrap().type_id())
            .and_then(|r| {
                r.data::<ReflectFromReflect>()
                    .and_then(|fr| fr.from_reflect(self.as_partial_reflect()))
                    .map(|fr| fr.into_partial_reflect())
            })
            .unwrap_or_else(|| self.to_dynamic())
    }
}

impl CloneReflect for DynamicEntity {
    fn clone_reflect(&self, registry: &TypeRegistry) -> Self {
        DynamicEntity {
            entity: self.entity,
            components: self.components.clone_reflect(registry),
        }
    }
}
