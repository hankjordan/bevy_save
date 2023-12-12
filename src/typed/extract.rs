use std::marker::PhantomData;

use bevy::{
    ecs::{
        component::Component,
        entity::{
            EntityMapper,
            MapEntities,
        },
        reflect::AppTypeRegistry,
        system::Resource,
        world::{
            EntityRef,
            EntityWorldMut,
            World,
        },
    },
    reflect::{
        serde::ReflectValueSerializer,
        Enum,
        FromReflect,
        Reflect,
        TypeRegistry,
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
    fn extract(world: &World, entity: &EntityRef) -> Self::Value;
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

pub struct DynamicValue<T> {
    value: Option<T>,
    registry: AppTypeRegistry,
}

impl<T> Extractable for Dynamic<T> {
    type Value = DynamicValue<T>;
}

impl<T: Component + FromReflect> ExtractComponent for Dynamic<T> {
    fn extract(world: &World, entity: &EntityRef) -> Self::Value {
        let value = entity
            .get::<T>()
            .map(|c| T::take_from_reflect(c.clone_value()).unwrap());

        DynamicValue {
            value,
            registry: world.resource::<AppTypeRegistry>().clone(),
        }
    }

    fn apply(value: &Self::Value, entity: &mut EntityWorldMut) {
        if let Some(value) = &value.value {
            entity.insert(T::take_from_reflect(value.clone_value()).unwrap());
        }
    }
}

impl<T: Resource + FromReflect> ExtractResource for Dynamic<T> {
    fn extract(world: &World) -> Self::Value {
        let value = world
            .get_resource::<T>()
            .map(|c| T::take_from_reflect(c.clone_value()).unwrap());

        DynamicValue {
            value,
            registry: world.resource::<AppTypeRegistry>().clone(),
        }
    }

    fn apply(value: &Self::Value, world: &mut World) {
        if let Some(value) = &value.value {
            world.insert_resource(T::take_from_reflect(value.clone_value()).unwrap());
        }
    }
}

impl<T: Reflect> ExtractSerialize for Dynamic<T> {
    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Value,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        if let Some(data) = &value.value {
            seq.serialize_element(&ReflectSerializer {
                registry: &value.registry.read(),
                value: data.as_reflect(),
            })?;
        } else {
            seq.serialize_element(&None::<()>)?;
        }

        Ok(())
    }
}

struct ReflectSerializer<'a> {
    value: &'a dyn Reflect,
    registry: &'a TypeRegistry,
}

impl<'a> Serialize for ReflectSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.value.reflect_ref() {
            bevy::reflect::ReflectRef::Struct(value) => {
                if serializer.is_human_readable() || value.field_len() == 0 {
                    let mut ser = serializer.serialize_map(Some(value.field_len()))?;

                    for (i, field) in value.iter_fields().enumerate() {
                        ser.serialize_entry(value.name_at(i).unwrap(), &Self {
                            registry: self.registry,
                            value: field,
                        })?;
                    }

                    ser.end()
                } else {
                    let mut ser = serializer.serialize_seq(Some(value.field_len()))?;

                    for field in value.iter_fields() {
                        ser.serialize_element(&Self {
                            registry: self.registry,
                            value: field,
                        })?;
                    }

                    ser.end()
                }
            }
            bevy::reflect::ReflectRef::TupleStruct(tuple) => {
                if tuple.field_len() > 1 {
                    let mut ser = serializer.serialize_seq(Some(tuple.field_len()))?;

                    for field in tuple.iter_fields() {
                        ser.serialize_element(&Self {
                            registry: self.registry,
                            value: field,
                        })?;
                    }

                    ser.end()
                } else {
                    Self {
                        value: tuple.field(0).unwrap(),
                        registry: self.registry,
                    }
                    .serialize(serializer)
                }
            }
            bevy::reflect::ReflectRef::Tuple(tuple) => {
                if tuple.field_len() > 0 {
                    let mut ser = serializer.serialize_seq(Some(tuple.field_len()))?;

                    for field in tuple.iter_fields() {
                        ser.serialize_element(&Self {
                            registry: self.registry,
                            value: field,
                        })?;
                    }

                    ser.end()
                } else {
                    serializer.serialize_unit()
                }
            }
            bevy::reflect::ReflectRef::List(list) => {
                let mut ser = serializer.serialize_seq(Some(list.len()))?;

                for item in list.iter() {
                    ser.serialize_element(&Self {
                        registry: self.registry,
                        value: item,
                    })?;
                }

                ser.end()
            }
            bevy::reflect::ReflectRef::Array(array) => {
                let mut ser = serializer.serialize_seq(Some(array.len()))?;

                for item in array.iter() {
                    ser.serialize_element(&Self {
                        registry: self.registry,
                        value: item,
                    })?;
                }

                ser.end()
            }
            bevy::reflect::ReflectRef::Map(map) => {
                let mut ser = serializer.serialize_map(Some(map.len()))?;

                for (key, value) in map.iter() {
                    ser.serialize_entry(
                        &Self {
                            registry: self.registry,
                            value: key,
                        },
                        &Self {
                            registry: self.registry,
                            value,
                        },
                    )?;
                }

                ser.end()
            }
            bevy::reflect::ReflectRef::Enum(value) => {
                struct Struct<'a>(&'a dyn Enum, &'a TypeRegistry);

                impl<'a> Serialize for Struct<'a> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        if serializer.is_human_readable() {
                            let mut ser = serializer.serialize_map(Some(self.0.field_len()))?;

                            for field in self.0.iter_fields() {
                                if let VariantField::Struct(name, value) = field {
                                    ser.serialize_entry(name, &ReflectSerializer {
                                        registry: self.1,
                                        value,
                                    })?;
                                }
                            }

                            ser.end()
                        } else {
                            let mut ser = serializer.serialize_seq(Some(self.0.field_len()))?;

                            for field in self.0.iter_fields() {
                                if let VariantField::Struct(_, value) = field {
                                    ser.serialize_element(&ReflectSerializer {
                                        registry: self.1,
                                        value,
                                    })?;
                                }
                            }

                            ser.end()
                        }
                    }
                }

                struct Tuple<'a>(&'a dyn Enum, &'a TypeRegistry);

                impl<'a> Serialize for Tuple<'a> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        let mut ser = serializer.serialize_seq(Some(self.0.field_len()))?;

                        for field in self.0.iter_fields() {
                            if let VariantField::Tuple(value) = field {
                                ser.serialize_element(&ReflectSerializer {
                                    registry: self.1,
                                    value,
                                })?;
                            }
                        }

                        ser.end()
                    }
                }

                match value.variant_type() {
                    bevy::reflect::VariantType::Struct => {
                        let mut ser = serializer.serialize_map(Some(1))?;
                        ser.serialize_entry(value.variant_name(), &Struct(value, self.registry))?;
                        ser.end()
                    }
                    bevy::reflect::VariantType::Tuple => {
                        let mut ser = serializer.serialize_map(Some(1))?;
                        ser.serialize_entry(value.variant_name(), &Tuple(value, self.registry))?;
                        ser.end()
                    }
                    bevy::reflect::VariantType::Unit => {
                        serializer.serialize_str(value.variant_name())
                    }
                }
            }
            bevy::reflect::ReflectRef::Value(value) => ReflectValueSerializer {
                value,
                registry: self.registry,
            }
            .serialize(serializer),
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
    fn extract(_: &World, entity: &EntityRef) -> Self::Value {
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
    fn extract(world: &World, entity: &EntityRef) -> Self::Value {
        E::extract(world, entity)
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
        if let Some(value) = &mut value.value {
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
    fn extract(_: &World, _: &EntityRef) -> Self::Value {}
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
    fn extract(world: &World, entity: &EntityRef) -> Self::Value {
        (T0::extract(world, entity), T1::extract(world, entity))
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
