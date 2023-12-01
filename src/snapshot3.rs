use std::marker::PhantomData;

use bevy::{
    ecs::world::EntityRef,
    prelude::*,
};
use serde::{
    de::Visitor,
    ser::{
        SerializeMap,
        SerializeSeq,
        SerializeStruct,
    },
    Deserialize,
    Serialize,
};

trait ExtractComponent {
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

trait ExtractResource {
    type Output;

    fn extract(world: &World) -> Self::Output;

    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error>;

    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error>;
}

struct Ext<T>(PhantomData<T>);

/// Necessary to serialize unit values correctly.
struct UnitSer<'a, T> {
    value: &'a T,
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

struct UnitDe<T> {
    value: T,
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
                    // SAFETY: T is zero-sized
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

impl<T: Component + Clone + Serialize + for<'de> Deserialize<'de>> ExtractComponent for Ext<T> {
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
        seq.next_element::<Option<UnitDe<T>>>().map(|e| e.flatten().map(|e| e.value))
    }
}

impl<T: Resource + Clone + Serialize + for<'de> Deserialize<'de>> ExtractResource for Ext<T> {
    type Output = Option<T>;

    fn extract(world: &World) -> Self::Output {
        world.get_resource::<T>().cloned()
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
        seq.next_element::<Option<UnitDe<T>>>().map(|e| e.flatten().map(|e| e.value))
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

    fn serialize<S: serde::ser::SerializeSeq>(_: &Self::Output, _: &mut S) -> Result<(), S::Error> {
        Ok(())
    }

    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(_: &mut D) -> Result<Self::Output, D::Error> {
        Ok(())
    }
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

struct Registry<C, R> {
    _marker: PhantomData<(C, R)>,
}

impl Registry<(), ()> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<C, R> Registry<C, R> {
    pub fn register_component<T: Component + Clone>(self) -> Registry<(C, Ext<T>), R> {
        Registry {
            _marker: PhantomData,
        }
    }

    pub fn register_resource<T: Resource + Clone>(self) -> Registry<C, (R, Ext<T>)> {
        Registry {
            _marker: PhantomData,
        }
    }
}

impl<C, R> Registry<C, R>
where
    C: ExtractComponent,
    R: ExtractResource,
{
    pub fn deserialize<'de, D: serde::de::Deserializer<'de>>(
        &self,
        de: D,
    ) -> Result<Snapshot<C, R>, D::Error> {
        Snapshot::<C, R>::deserialize(de)
    }

    pub fn extract(&self, world: &World) -> Snapshot<C, R> {
        Snapshot {
            entities: Entities(
                world
                    .iter_entities()
                    .map(|e| (e.id(), Components(C::extract(&e))))
                    .collect(),
            ),
            resources: Resources(R::extract(world)),
        }
    }
}

#[test]
fn test_snapshot() {
    #[derive(Component, Clone, Serialize, Deserialize)]
    struct ExampleComponent {
        name: String,
    }

    #[derive(Component, Clone, Serialize, Deserialize)]
    struct OtherComponent;

    #[derive(Resource, Clone, Serialize, Deserialize)]
    struct SimpleResource {
        data: u32,
    }

    let mut app = App::new();
    let world = &mut app.world;

    world.spawn(ExampleComponent {
        name: "First".into(),
    });
    world.spawn((
        ExampleComponent {
            name: "Second".into(),
        },
        OtherComponent,
    ));
    world.spawn(OtherComponent);
    world.spawn(OtherComponent);

    world.insert_resource(SimpleResource { data: 42 });

    let registry = Registry::new()
        .register_component::<ExampleComponent>()
        .register_component::<OtherComponent>()
        .register_resource::<SimpleResource>();

    let snapshot = registry.extract(world);

    let mut buf = Vec::new();

    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);

    snapshot.serialize(&mut ser).unwrap();

    let output = std::str::from_utf8(&buf).unwrap();
    let expected = r#"{
    "entities": {
        "0": [
            {
                "name": "First"
            },
            null
        ],
        "1": [
            {
                "name": "Second"
            },
            {}
        ],
        "2": [
            null,
            {}
        ],
        "3": [
            null,
            {}
        ]
    },
    "resources": [
        {
            "data": 42
        }
    ]
}"#;

    assert_eq!(output, expected);

    let mut de = serde_json::Deserializer::from_str(output);

    let snapshot = registry.deserialize(&mut de).unwrap();

    let mut buf = Vec::new();

    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);

    snapshot.serialize(&mut ser).unwrap();

    let output = std::str::from_utf8(&buf).unwrap();

    assert_eq!(output, expected);
}

struct Snapshot<C: ExtractComponent, R: ExtractResource> {
    entities: Entities<C>,
    resources: Resources<R>,
}

impl<C: ExtractComponent, R: ExtractResource> Serialize for Snapshot<C, R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ser = serializer.serialize_struct("Snapshot", 2)?;
        ser.serialize_field("entities", &self.entities)?;
        ser.serialize_field("resources", &self.resources)?;
        ser.end()
    }
}

impl<'de, C: ExtractComponent, R: ExtractResource> Deserialize<'de> for Snapshot<C, R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Fields {
            Entities,
            Resources,
        }

        struct SnapshotVisitor<C, R>(PhantomData<(C, R)>);

        impl<'de, C: ExtractComponent, R: ExtractResource> Visitor<'de> for SnapshotVisitor<C, R> {
            type Value = Snapshot<C, R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("snapshot struct")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let entities = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

                let resources = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

                Ok(Snapshot {
                    entities,
                    resources,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut entities = None;
                let mut resources = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Fields::Entities => {
                            if entities.is_some() {
                                return Err(serde::de::Error::duplicate_field("entities"));
                            }
                            entities = Some(map.next_value()?);
                        }
                        Fields::Resources => {
                            if resources.is_some() {
                                return Err(serde::de::Error::duplicate_field("resources"));
                            }
                            resources = Some(map.next_value()?);
                        }
                    }
                }

                let entities =
                    entities.ok_or_else(|| serde::de::Error::missing_field("entities"))?;
                let resources =
                    resources.ok_or_else(|| serde::de::Error::missing_field("resources"))?;

                Ok(Snapshot {
                    entities,
                    resources,
                })
            }
        }

        const FIELDS: &[&str] = &["entities", "resources"];
        deserializer.deserialize_struct("Snapshot", FIELDS, SnapshotVisitor(PhantomData))
    }
}

struct Entities<C: ExtractComponent>(Vec<(Entity, Components<C>)>);

impl<C: ExtractComponent> Serialize for Entities<C> {
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

impl<'de, C: ExtractComponent> Deserialize<'de> for Entities<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EntitiesVisitor<C>(PhantomData<C>);

        impl<'de, C: ExtractComponent> Visitor<'de> for EntitiesVisitor<C> {
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

struct Components<C: ExtractComponent>(C::Output);

impl<C: ExtractComponent> Serialize for Components<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        C::serialize(&self.0, &mut seq)?;
        seq.end()
    }
}

impl<'de, C: ExtractComponent> Deserialize<'de> for Components<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ComponentsVisitor<C>(PhantomData<C>);

        impl<'de, C: ExtractComponent> Visitor<'de> for ComponentsVisitor<C> {
            type Value = Components<C>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of extracted components")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                Ok(Components(C::deserialize(&mut seq)?))
            }
        }

        deserializer.deserialize_seq(ComponentsVisitor(PhantomData))
    }
}

struct Resources<R: ExtractResource>(R::Output);

impl<R: ExtractResource> Serialize for Resources<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        R::serialize(&self.0, &mut seq)?;
        seq.end()
    }
}

impl<'de, R: ExtractResource> Deserialize<'de> for Resources<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ResourcesVisitor<R>(PhantomData<R>);

        impl<'de, R: ExtractResource> Visitor<'de> for ResourcesVisitor<R> {
            type Value = Resources<R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of extracted resources")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                Ok(Resources(R::deserialize(&mut seq)?))
            }
        }

        deserializer.deserialize_seq(ResourcesVisitor(PhantomData))
    }
}
