use bevy::reflect::{
    PartialReflect,
    ReflectFromReflect,
    TypeRegistry,
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
    value: &dyn PartialReflect,
    registry: &TypeRegistry,
) -> Box<dyn PartialReflect> {
    value.reflect_clone().map_or_else(
        |_| {
            value
                .get_represented_type_info()
                .and_then(|i| registry.get(i.type_id()))
                .and_then(|r| r.data::<ReflectFromReflect>())
                .and_then(|fr| fr.from_reflect(value))
                .map_or_else(|| value.to_dynamic(), PartialReflect::into_partial_reflect)
        },
        PartialReflect::into_partial_reflect,
    )
}
