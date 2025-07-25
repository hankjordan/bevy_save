use bevy::{
    reflect::{
        PartialReflect,
        ReflectFromReflect,
        TypeRegistry,
    },
    scene::DynamicEntity,
};

/// Attempts to clone a [`PartialReflect`] value using various methods.
///
/// This first attempts to clone via [`PartialReflect::reflect_clone`].
/// then falls back to [`ReflectFromReflect::from_reflect`],
/// and finally [`PartialReflect::to_dynamic`] if the first two methods fail.
///
/// This helps ensure that the original type and type data is retained,
/// and only returning a dynamic type if all other methods fail.
pub fn clone_reflect_value(
    value: &(impl PartialReflect + ?Sized),
    registry: &TypeRegistry,
) -> Box<dyn PartialReflect> {
    value.reflect_clone().map_or_else(
        |_| {
            value
                .get_represented_type_info()
                .and_then(|i| registry.get(i.type_id()))
                .and_then(|r| r.data::<ReflectFromReflect>())
                .and_then(|fr| fr.from_reflect(value.as_partial_reflect()))
                .map_or_else(|| value.to_dynamic(), PartialReflect::into_partial_reflect)
        },
        PartialReflect::into_partial_reflect,
    )
}

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
