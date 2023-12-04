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
        World,
    },
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    UnitDe,
    UnitSer,
};

pub trait ExtractComponent {
    type Output;

    fn extract(entity: &EntityRef) -> Self::Output;

    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error>;

    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error>;
}

pub trait ExtractResource {
    type Output;

    fn extract(world: &World) -> Self::Output;
    fn apply(value: &Self::Output, world: &mut World);

    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error>;

    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error>;
}

pub trait ExtractMapEntities {
    type Value;
    fn map_entities(value: &mut Self::Value, entity_mapper: &mut EntityMapper);
}

pub struct Extract<T>(PhantomData<T>);

impl<T: Component + Clone + Serialize + for<'de> Deserialize<'de>> ExtractComponent for Extract<T> {
    type Output = Option<T>;

    fn extract(entity: &EntityRef) -> Self::Output {
        entity.get::<T>().cloned()
    }

    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(&value.as_ref().map(UnitSer::new))
    }

    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error> {
        seq.next_element::<Option<UnitDe<T>>>()
            .map(|e| e.flatten().map(|e| e.value))
    }
}

impl<T: Resource + Clone + Serialize + for<'de> Deserialize<'de>> ExtractResource for Extract<T> {
    type Output = Option<T>;

    fn extract(world: &World) -> Self::Output {
        world.get_resource::<T>().cloned()
    }

    fn apply(value: &Self::Output, world: &mut World) {
        if let Some(value) = value {
            world.insert_resource(value.clone());
        }
    }

    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(&value.as_ref().map(UnitSer::new))
    }

    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error> {
        seq.next_element::<Option<UnitDe<T>>>()
            .map(|e| e.flatten().map(|e| e.value))
    }
}

impl<T: MapEntities> ExtractMapEntities for Extract<T> {
    type Value = T;

    fn map_entities(value: &mut Self::Value, entity_mapper: &mut EntityMapper) {
        value.map_entities(entity_mapper);
    }
}

impl ExtractComponent for () {
    type Output = ();

    fn extract(_: &EntityRef) -> Self::Output {}

    fn serialize<S: serde::ser::SerializeSeq>(_: &Self::Output, _: &mut S) -> Result<(), S::Error> {
        Ok(())
    }

    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(_: &mut D) -> Result<Self::Output, D::Error> {
        Ok(())
    }
}

impl ExtractResource for () {
    type Output = ();

    fn extract(_: &World) -> Self::Output {}

    fn apply(_: &Self::Output, _: &mut World) {}

    fn serialize<S: serde::ser::SerializeSeq>(_: &Self::Output, _: &mut S) -> Result<(), S::Error> {
        Ok(())
    }

    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(_: &mut D) -> Result<Self::Output, D::Error> {
        Ok(())
    }
}

impl ExtractMapEntities for () {
    type Value = ();

    fn map_entities(_: &mut Self::Value, _: &mut EntityMapper) {}
}

impl<T0: ExtractComponent, T1: ExtractComponent> ExtractComponent for (T0, T1) {
    type Output = (T0::Output, T1::Output);

    fn extract(entity: &EntityRef) -> Self::Output {
        (T0::extract(entity), T1::extract(entity))
    }

    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        T0::serialize(&value.0, seq)?;
        T1::serialize(&value.1, seq)?;
        Ok(())
    }

    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error> {
        Ok((T0::deserialize(seq)?, T1::deserialize(seq)?))
    }
}

impl<T0: ExtractResource, T1: ExtractResource> ExtractResource for (T0, T1) {
    type Output = (T0::Output, T1::Output);

    fn extract(world: &World) -> Self::Output {
        (T0::extract(world), T1::extract(world))
    }

    fn apply(value: &Self::Output, world: &mut World) {
        T0::apply(&value.0, world);
        T1::apply(&value.1, world);
    }

    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        T0::serialize(&value.0, seq)?;
        T1::serialize(&value.1, seq)?;
        Ok(())
    }

    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error> {
        Ok((T0::deserialize(seq)?, T1::deserialize(seq)?))
    }
}

impl<T0: ExtractMapEntities, T1: ExtractMapEntities> ExtractMapEntities for (T0, T1) {
    type Value = (T0::Value, T1::Value);

    fn map_entities(value: &mut Self::Value, entity_mapper: &mut EntityMapper) {
        T0::map_entities(&mut value.0, entity_mapper);
        T1::map_entities(&mut value.1, entity_mapper);
    }
}
