use std::marker::PhantomData;

use serde::{
    de::Visitor,
    ser::{
        SerializeMap,
        SerializeSeq,
    },
    Deserialize,
    Serialize,
};

use crate::typed::{
    extract::{
        ExtractDeserialize,
        ExtractSerialize,
    },
    snapshot::{
        Entities,
        Extracted,
    },
};

impl<C> Serialize for Entities<C>
where
    C: ExtractSerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;

        for (entity, components) in &self.0 {
            map.serialize_entry(entity, components)?;
        }

        map.end()
    }
}

impl<'de, C> Deserialize<'de> for Entities<C>
where
    C: ExtractDeserialize,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EntitiesVisitor<C>(PhantomData<C>);

        impl<'de, C> Visitor<'de> for EntitiesVisitor<C>
        where
            C: ExtractDeserialize,
        {
            type Value = Entities<C>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of extracted entities")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut entities = Vec::new();

                while let Some(entry) = map.next_entry()? {
                    entities.push(entry);
                }

                Ok(Entities(entities))
            }
        }

        deserializer.deserialize_map(EntitiesVisitor(PhantomData))
    }
}

impl<E> Serialize for Extracted<E>
where
    E: ExtractSerialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        E::serialize(&self.0, &mut seq)?;
        seq.end()
    }
}

impl<'de, E> Deserialize<'de> for Extracted<E>
where
    E: ExtractDeserialize,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ExtractedVisitor<C>(PhantomData<C>);

        impl<'de, E> Visitor<'de> for ExtractedVisitor<E>
        where
            E: ExtractDeserialize,
        {
            type Value = Extracted<E>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of extracted values")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                Ok(Extracted(E::deserialize(&mut seq)?))
            }
        }

        deserializer.deserialize_seq(ExtractedVisitor(PhantomData))
    }
}

// Unit types ---------------------------------------------------------------------------------------------------------

pub(crate) struct UnitSer<'a, T> {
    pub(crate) value: &'a T,
}

impl<'a, T> UnitSer<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self { value }
    }
}

impl<'a, T: Serialize> Serialize for UnitSer<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if std::mem::size_of::<T>() == 0 {
            let seq = serializer.serialize_map(Some(0))?;
            seq.end()
        } else {
            self.value.serialize(serializer)
        }
    }
}

pub(crate) struct UnitDe<T> {
    pub(crate) value: T,
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for UnitDe<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if std::mem::size_of::<T>() == 0 {
            struct UnitDeVisitor<T>(PhantomData<T>);

            impl<'de, T: Deserialize<'de>> Visitor<'de> for UnitDeVisitor<T> {
                type Value = T;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("an empty map")
                }

                fn visit_map<A>(self, _: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    // SAFETY: T is Unit value
                    #[allow(clippy::uninit_assumed_init)]
                    Ok(unsafe { std::mem::MaybeUninit::<T>::uninit().assume_init() })
                }
            }

            deserializer.deserialize_map(UnitDeVisitor(PhantomData))
        } else {
            T::deserialize(deserializer).map(|value| UnitDe { value })
        }
    }
}
