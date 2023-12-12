use std::marker::PhantomData;

use bevy::{
    ecs::{
        component::Component,
        entity::{
            EntityMapper,
            MapEntities,
        },
        system::Resource,
        world::{
            EntityRef,
            EntityWorldMut,
            World,
        },
    },
    reflect::{
        Enum,
        FromReflect,
        FromType,
        Reflect,
        ReflectSerialize,
        TypePath,
        VariantField,
    },
};
use serde::{
    ser::{
        SerializeMap,
        SerializeSeq,
    },
    Deserialize,
    Serialize,
};

use crate::typed::serde::{
    UnitDe,
    UnitSer,
};

pub(crate) trait Extractable {
    type Value;
}

pub trait ExtractComponent: Extractable {
    fn extract(entity: &EntityRef) -> Self::Value;
    fn apply(value: &Self::Value, entity: &mut EntityWorldMut);
}

pub trait ExtractResource: Extractable {
    fn extract(world: &World) -> Self::Value;
    fn apply(value: &Self::Value, world: &mut World);
}

pub trait ExtractSerialize: Extractable {
    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Value,
        seq: &mut S,
    ) -> Result<(), S::Error>;
}

pub trait ExtractDeserialize: Extractable {
    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(seq: &mut D)
        -> Result<Self::Value, D::Error>;
}

pub trait ExtractMapEntities: Extractable {
    fn map_entities(value: &mut Self::Value, entity_mapper: &mut EntityMapper);
}

pub struct Dynamic<T>(PhantomData<T>);

impl<T> Extractable for Dynamic<T> {
    type Value = Option<T>;
}

impl<T: Component + FromReflect> ExtractComponent for Dynamic<T> {
    fn extract(entity: &EntityRef) -> Self::Value {
        entity
            .get::<T>()
            .map(|c| T::take_from_reflect(c.clone_value()).unwrap())
    }

    fn apply(value: &Self::Value, entity: &mut EntityWorldMut) {
        if let Some(value) = value {
            entity.insert(T::take_from_reflect(value.clone_value()).unwrap());
        }
    }
}

impl<T: Resource + FromReflect> ExtractResource for Dynamic<T> {
    fn extract(world: &World) -> Self::Value {
        world
            .get_resource::<T>()
            .map(|c| T::take_from_reflect(c.clone_value()).unwrap())
    }

    fn apply(value: &Self::Value, world: &mut World) {
        if let Some(value) = value {
            world.insert_resource(T::take_from_reflect(value.clone_value()).unwrap());
        }
    }
}

impl<T: Reflect> ExtractSerialize for Dynamic<T> {
    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Value,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        if let Some(value) = value {
            seq.serialize_element(&ReflectSerializer(value.as_reflect()))?;
        }

        Ok(())
    }
}

struct ReflectSerializer<'a>(&'a dyn Reflect);

impl<'a> Serialize for ReflectSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0.reflect_ref() {
            bevy::reflect::ReflectRef::Struct(value) => {
                let mut ser = serializer.serialize_map(Some(value.field_len()))?;

                for (i, field) in value.iter_fields().enumerate() {
                    ser.serialize_entry(value.name_at(i).unwrap(), &Self(field))?;
                }

                ser.end()
            }
            bevy::reflect::ReflectRef::TupleStruct(tuple) => {
                let mut ser = serializer.serialize_seq(Some(tuple.field_len()))?;

                for field in tuple.iter_fields() {
                    ser.serialize_element(&Self(field))?;
                }

                ser.end()
            }
            bevy::reflect::ReflectRef::Tuple(tuple) => {
                let mut ser = serializer.serialize_seq(Some(tuple.field_len()))?;

                for field in tuple.iter_fields() {
                    ser.serialize_element(&Self(field))?;
                }

                ser.end()
            }
            bevy::reflect::ReflectRef::List(list) => {
                let mut ser = serializer.serialize_seq(Some(list.len()))?;

                for item in list.iter() {
                    ser.serialize_element(&Self(item))?;
                }

                ser.end()
            }
            bevy::reflect::ReflectRef::Array(array) => {
                let mut ser = serializer.serialize_seq(Some(array.len()))?;

                for item in array.iter() {
                    ser.serialize_element(&Self(item))?;
                }

                ser.end()
            }
            bevy::reflect::ReflectRef::Map(map) => {
                let mut ser = serializer.serialize_map(Some(map.len()))?;

                for (key, value) in map.iter() {
                    ser.serialize_entry(&Self(key), &Self(value))?;
                }

                ser.end()
            }
            bevy::reflect::ReflectRef::Enum(value) => {
                struct Struct<'a>(&'a dyn Enum);

                impl<'a> Serialize for Struct<'a> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        let mut ser = serializer.serialize_map(Some(self.0.field_len()))?;

                        for field in self.0.iter_fields() {
                            if let VariantField::Struct(name, value) = field {
                                ser.serialize_entry(name, &ReflectSerializer(value))?;
                            }
                        }

                        ser.end()
                    }
                }

                struct Tuple<'a>(&'a dyn Enum);

                impl<'a> Serialize for Tuple<'a> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        let mut ser = serializer.serialize_seq(Some(self.0.field_len()))?;

                        for field in self.0.iter_fields() {
                            if let VariantField::Tuple(value) = field {
                                ser.serialize_element(&ReflectSerializer(value))?;
                            }
                        }

                        ser.end()
                    }
                }

                match value.variant_type() {
                    bevy::reflect::VariantType::Struct => {
                        let mut ser = serializer.serialize_map(Some(2))?;
                        ser.serialize_entry(value.variant_name(), &Struct(value))?;
                        ser.end()
                    }
                    bevy::reflect::VariantType::Tuple => {
                        let mut ser = serializer.serialize_map(Some(2))?;
                        ser.serialize_entry(value.variant_name(), &Tuple(value))?;
                        ser.end()
                    }
                    bevy::reflect::VariantType::Unit => {
                        serializer.serialize_str(value.variant_name())
                    }
                }
            }
            bevy::reflect::ReflectRef::Value(value) => {
                // Primitive, attempt to serialize directly
                match value
                    .get_represented_type_info()
                    .map(|t| t.type_path())
                    .expect("Attempted to serialize a non-primitive type as a primitive type")
                {
                    "bool" => todo!(),
                    "char" => todo!(),
                    "f32" => todo!(),
                    "f64" => todo!(),
                    "i8" => todo!(),
                    "i16" => todo!(),
                    "i32" => todo!(),
                    "i64" => todo!(),
                    "i128" => todo!(),
                    "isize" => todo!(),
                    "u8" => todo!(),
                    "u16" => todo!(),
                    "u32" => todo!(),
                    "u64" => todo!(),
                    "u128" => todo!(),
                    "usize" => todo!(),
                    _ => unimplemented!(),
                }
            }
        }
    }
}

impl<T: Reflect> ExtractDeserialize for Dynamic<T> {
    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Value, D::Error> {
        // seq.next_element::<Option<UnitDe<T>>>()
        //     .map(|e| e.flatten().map(|e| e.value))
        todo!()
    }
}

impl<T> ExtractMapEntities for Dynamic<T> {
    fn map_entities(_: &mut Self::Value, _: &mut EntityMapper) {}
}

pub struct Typed<T>(PhantomData<T>);

impl<T> Extractable for Typed<T> {
    type Value = Option<T>;
}

impl<T: Component + Clone> ExtractComponent for Typed<T> {
    fn extract(entity: &EntityRef) -> Self::Value {
        entity.get::<T>().cloned()
    }

    fn apply(value: &Self::Value, entity: &mut EntityWorldMut) {
        if let Some(value) = value {
            entity.insert(value.clone());
        }
    }
}

impl<T: Resource + Clone> ExtractResource for Typed<T> {
    fn extract(world: &World) -> Self::Value {
        world.get_resource::<T>().cloned()
    }

    fn apply(value: &Self::Value, world: &mut World) {
        if let Some(value) = value {
            world.insert_resource(value.clone());
        }
    }
}

impl<T: Serialize> ExtractSerialize for Typed<T> {
    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Value,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(&value.as_ref().map(UnitSer::new))
    }
}

impl<T: for<'de> Deserialize<'de>> ExtractDeserialize for Typed<T> {
    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Value, D::Error> {
        seq.next_element::<Option<UnitDe<T>>>()
            .map(|e| e.flatten().map(|e| e.value))
    }
}

impl<T> ExtractMapEntities for Typed<T> {
    fn map_entities(_: &mut Self::Value, _: &mut EntityMapper) {}
}

pub struct Mapped<E>(PhantomData<E>);

impl<E: Extractable> Extractable for Mapped<E> {
    type Value = E::Value;
}

impl<E: ExtractComponent> ExtractComponent for Mapped<E> {
    fn extract(entity: &EntityRef) -> Self::Value {
        E::extract(entity)
    }

    fn apply(value: &Self::Value, entity: &mut EntityWorldMut) {
        E::apply(value, entity);
    }
}

impl<E: ExtractResource> ExtractResource for Mapped<E> {
    fn extract(world: &World) -> Self::Value {
        E::extract(world)
    }

    fn apply(value: &Self::Value, world: &mut World) {
        E::apply(value, world);
    }
}

impl<E: ExtractSerialize> ExtractSerialize for Mapped<E> {
    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Value,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        E::serialize(value, seq)
    }
}

impl<E: ExtractDeserialize> ExtractDeserialize for Mapped<E> {
    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Value, D::Error> {
        E::deserialize(seq)
    }
}

impl<T: MapEntities> ExtractMapEntities for Mapped<Dynamic<T>> {
    fn map_entities(value: &mut Self::Value, entity_mapper: &mut EntityMapper) {
        if let Some(value) = value {
            value.map_entities(entity_mapper);
        }
    }
}

impl<T: MapEntities> ExtractMapEntities for Mapped<Typed<T>> {
    fn map_entities(value: &mut Self::Value, entity_mapper: &mut EntityMapper) {
        if let Some(value) = value {
            value.map_entities(entity_mapper);
        }
    }
}

impl Extractable for () {
    type Value = ();
}

impl ExtractComponent for () {
    fn extract(_: &EntityRef) -> Self::Value {}
    fn apply(_: &Self::Value, _: &mut EntityWorldMut) {}
}

impl ExtractResource for () {
    fn extract(_: &World) -> Self::Value {}
    fn apply(_: &Self::Value, _: &mut World) {}
}

impl ExtractSerialize for () {
    fn serialize<S: serde::ser::SerializeSeq>(_: &Self::Value, _: &mut S) -> Result<(), S::Error> {
        Ok(())
    }
}

impl ExtractDeserialize for () {
    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(_: &mut D) -> Result<Self::Value, D::Error> {
        Ok(())
    }
}

impl ExtractMapEntities for () {
    fn map_entities(_: &mut Self::Value, _: &mut EntityMapper) {}
}

impl<T0: Extractable, T1: Extractable> Extractable for (T0, T1) {
    type Value = (T0::Value, T1::Value);
}

impl<T0: ExtractComponent, T1: ExtractComponent> ExtractComponent for (T0, T1) {
    fn extract(entity: &EntityRef) -> Self::Value {
        (T0::extract(entity), T1::extract(entity))
    }

    fn apply(value: &Self::Value, entity: &mut EntityWorldMut) {
        T0::apply(&value.0, entity);
        T1::apply(&value.1, entity);
    }
}

impl<T0: ExtractResource, T1: ExtractResource> ExtractResource for (T0, T1) {
    fn extract(world: &World) -> Self::Value {
        (T0::extract(world), T1::extract(world))
    }

    fn apply(value: &Self::Value, world: &mut World) {
        T0::apply(&value.0, world);
        T1::apply(&value.1, world);
    }
}

impl<T0: ExtractSerialize, T1: ExtractSerialize> ExtractSerialize for (T0, T1) {
    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Value,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        T0::serialize(&value.0, seq)?;
        T1::serialize(&value.1, seq)?;
        Ok(())
    }
}

impl<T0: ExtractDeserialize, T1: ExtractDeserialize> ExtractDeserialize for (T0, T1) {
    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Value, D::Error> {
        Ok((T0::deserialize(seq)?, T1::deserialize(seq)?))
    }
}

impl<T0: ExtractMapEntities, T1: ExtractMapEntities> ExtractMapEntities for (T0, T1) {
    fn map_entities(value: &mut Self::Value, entity_mapper: &mut EntityMapper) {
        T0::map_entities(&mut value.0, entity_mapper);
        T1::map_entities(&mut value.1, entity_mapper);
    }
}
