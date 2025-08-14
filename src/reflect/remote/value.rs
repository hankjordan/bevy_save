use bevy::reflect::{
    FromReflect,
    FromType,
    GetTypeRegistration,
    OpaqueInfo,
    PartialReflect,
    Reflect,
    ReflectFromPtr,
    ReflectFromReflect,
    ReflectRemote,
    TypeInfo,
    TypePath,
    TypeRegistration,
    Typed,
    utility::NonGenericTypeInfoCell,
};

/// Equivalent to [`Box<dyn PartialReflect>`], but implements [`Reflect`].
#[repr(transparent)]
#[derive(TypePath)]
#[type_path = "bevy_save"]
pub struct DynamicValue(pub Box<dyn PartialReflect>);

impl From<Box<dyn PartialReflect>> for DynamicValue {
    fn from(value: Box<dyn PartialReflect>) -> Self {
        // SAFETY: DynamicValue and Box<dyn PartialReflect> are equivalent
        unsafe { std::mem::transmute(value) }
    }
}

impl From<DynamicValue> for Box<dyn PartialReflect> {
    fn from(value: DynamicValue) -> Self {
        // SAFETY: DynamicValue and Box<dyn PartialReflect> are equivalent
        unsafe { std::mem::transmute(value) }
    }
}

impl From<&Box<dyn PartialReflect>> for &DynamicValue {
    fn from(value: &Box<dyn PartialReflect>) -> Self {
        // SAFETY: DynamicValue and Box<dyn PartialReflect> are equivalent
        unsafe { &*std::ptr::from_ref(value).cast() }
    }
}

impl From<&DynamicValue> for &Box<dyn PartialReflect> {
    fn from(value: &DynamicValue) -> Self {
        // SAFETY: DynamicValue and Box<dyn PartialReflect> are equivalent
        unsafe { &*std::ptr::from_ref(value).cast() }
    }
}

impl From<&mut Box<dyn PartialReflect>> for &mut DynamicValue {
    fn from(value: &mut Box<dyn PartialReflect>) -> Self {
        // SAFETY: DynamicValue and Box<dyn PartialReflect> are equivalent
        unsafe { &mut *std::ptr::from_mut(value).cast() }
    }
}

impl From<&mut DynamicValue> for &mut Box<dyn PartialReflect> {
    fn from(value: &mut DynamicValue) -> Self {
        // SAFETY: DynamicValue and Box<dyn PartialReflect> are equivalent
        unsafe { &mut *std::ptr::from_mut(value).cast() }
    }
}

impl std::fmt::Debug for DynamicValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.debug(f)
    }
}

impl std::ops::Deref for DynamicValue {
    type Target = Box<dyn PartialReflect>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DynamicValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ReflectRemote for DynamicValue {
    type Remote = Box<dyn PartialReflect>;

    fn as_remote(&self) -> &Self::Remote {
        self.into()
    }

    fn as_remote_mut(&mut self) -> &mut Self::Remote {
        self.into()
    }

    fn into_remote(self) -> Self::Remote {
        self.into()
    }

    fn as_wrapper(remote: &Self::Remote) -> &Self {
        remote.into()
    }

    fn as_wrapper_mut(remote: &mut Self::Remote) -> &mut Self {
        remote.into()
    }

    fn into_wrapper(remote: Self::Remote) -> Self {
        remote.into()
    }
}

impl PartialReflect for DynamicValue {
    fn get_represented_type_info(&self) -> Option<&'static bevy::reflect::TypeInfo> {
        self.0.get_represented_type_info()
    }

    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
        self.0.into_partial_reflect()
    }

    fn as_partial_reflect(&self) -> &dyn PartialReflect {
        self.0.as_partial_reflect()
    }

    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
        self.0.as_partial_reflect_mut()
    }

    fn try_into_reflect(
        self: Box<Self>,
    ) -> std::result::Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
        self.0.try_into_reflect()
    }

    fn try_as_reflect(&self) -> Option<&dyn Reflect> {
        self.0.try_as_reflect()
    }

    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
        self.0.try_as_reflect_mut()
    }

    fn try_apply(
        &mut self,
        value: &dyn PartialReflect,
    ) -> std::result::Result<(), bevy::reflect::ApplyError> {
        self.0.try_apply(value)
    }

    fn reflect_ref(&self) -> bevy::reflect::ReflectRef {
        self.0.reflect_ref()
    }

    fn reflect_mut(&mut self) -> bevy::reflect::ReflectMut {
        self.0.reflect_mut()
    }

    fn reflect_owned(self: Box<Self>) -> bevy::reflect::ReflectOwned {
        self.0.reflect_owned()
    }

    fn apply(&mut self, value: &dyn PartialReflect) {
        self.0.apply(value);
    }

    fn reflect_kind(&self) -> bevy::reflect::ReflectKind {
        self.0.reflect_kind()
    }

    fn clone_value(&self) -> Box<dyn PartialReflect> {
        #[expect(deprecated)]
        self.0.clone_value()
    }

    fn to_dynamic(&self) -> Box<dyn PartialReflect> {
        self.0.to_dynamic()
    }

    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, bevy::reflect::ReflectCloneError> {
        self.0.reflect_clone()
    }

    fn reflect_hash(&self) -> Option<u64> {
        self.0.reflect_hash()
    }

    fn reflect_partial_eq(&self, value: &dyn PartialReflect) -> Option<bool> {
        self.0.reflect_partial_eq(value)
    }

    fn debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.debug(f)
    }

    fn is_dynamic(&self) -> bool {
        self.0.is_dynamic()
    }
}

impl Reflect for DynamicValue {
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self.0
            .try_into_reflect()
            .expect("Underlying type does not implement `Reflect`")
            .into_any()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        &self.0 as _
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        &mut self.0 as _
    }

    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
        self.0
            .try_into_reflect()
            .expect("Underlying type does not implement `Reflect`")
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self.0
            .try_as_reflect()
            .expect("Underlying type does not implement `Reflect`")
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self.0
            .try_as_reflect_mut()
            .expect("Underlying type does not implement `Reflect`")
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> std::result::Result<(), Box<dyn Reflect>> {
        self.0 = value.into_partial_reflect();
        Ok(())
    }
}

impl Typed for DynamicValue {
    fn type_info() -> &'static bevy::reflect::TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl GetTypeRegistration for DynamicValue {
    fn get_type_registration() -> bevy::reflect::TypeRegistration {
        let mut registration = TypeRegistration::of::<Self>();
        registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
        registration.insert::<ReflectFromReflect>(FromType::<Self>::from_type());
        registration
    }
}

impl FromReflect for DynamicValue {
    fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
        Some(Self(reflect.reflect_clone().map_or_else(
            |_| reflect.to_dynamic(),
            PartialReflect::into_partial_reflect,
        )))
    }
}
