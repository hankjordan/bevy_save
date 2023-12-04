use std::marker::PhantomData;

use bevy::ecs::{
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
};
use serde::{
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

pub(crate) trait ExtractComponent: Extractable {
    fn extract(entity: &EntityRef) -> Self::Value;
    fn apply(value: &Self::Value, entity: &mut EntityWorldMut);
}

pub(crate) trait ExtractResource: Extractable {
    fn extract(world: &World) -> Self::Value;
    fn apply(value: &Self::Value, world: &mut World);
}

pub(crate) trait ExtractSerialize: Extractable {
    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Value,
        seq: &mut S,
    ) -> Result<(), S::Error>;
}

pub(crate) trait ExtractDeserialize: Extractable {
    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(seq: &mut D)
        -> Result<Self::Value, D::Error>;
}

pub(crate) trait ExtractMapEntities: Extractable {
    fn map_entities(value: &mut Self::Value, entity_mapper: &mut EntityMapper);
}

pub(crate) struct Extract<T>(PhantomData<T>);

impl<T> Extractable for Extract<T> {
    type Value = Option<T>;
}

impl<T: Component + Clone> ExtractComponent for Extract<T> {
    fn extract(entity: &EntityRef) -> Self::Value {
        entity.get::<T>().cloned()
    }

    fn apply(value: &Self::Value, entity: &mut EntityWorldMut) {
        if let Some(value) = value {
            entity.insert(value.clone());
        }
    }
}

impl<T: Resource + Clone> ExtractResource for Extract<T> {
    fn extract(world: &World) -> Self::Value {
        world.get_resource::<T>().cloned()
    }

    fn apply(value: &Self::Value, world: &mut World) {
        if let Some(value) = value {
            world.insert_resource(value.clone());
        }
    }
}

impl<T: Serialize> ExtractSerialize for Extract<T> {
    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Value,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(&value.as_ref().map(UnitSer::new))
    }
}

impl<T: for<'de> Deserialize<'de>> ExtractDeserialize for Extract<T> {
    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Value, D::Error> {
        seq.next_element::<Option<UnitDe<T>>>()
            .map(|e| e.flatten().map(|e| e.value))
    }
}

impl<T> ExtractMapEntities for Extract<T> {
    fn map_entities(_: &mut Self::Value, _: &mut EntityMapper) {}
}

pub(crate) struct ExtractMap<T>(PhantomData<T>);

impl<T> Extractable for ExtractMap<T> {
    type Value = Option<T>;
}

impl<T: Component + Clone> ExtractComponent for ExtractMap<T> {
    fn extract(entity: &EntityRef) -> Self::Value {
        entity.get::<T>().cloned()
    }

    fn apply(value: &Self::Value, entity: &mut EntityWorldMut) {
        if let Some(value) = value {
            entity.insert(value.clone());
        }
    }
}

impl<T: Resource + Clone> ExtractResource for ExtractMap<T> {
    fn extract(world: &World) -> Self::Value {
        world.get_resource::<T>().cloned()
    }

    fn apply(value: &Self::Value, world: &mut World) {
        if let Some(value) = value {
            world.insert_resource(value.clone());
        }
    }
}

impl<T: Serialize> ExtractSerialize for ExtractMap<T> {
    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Value,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(&value.as_ref().map(UnitSer::new))
    }
}

impl<T: for<'de> Deserialize<'de>> ExtractDeserialize for ExtractMap<T> {
    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Value, D::Error> {
        seq.next_element::<Option<UnitDe<T>>>()
            .map(|e| e.flatten().map(|e| e.value))
    }
}

impl<T: MapEntities> ExtractMapEntities for ExtractMap<T> {
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
