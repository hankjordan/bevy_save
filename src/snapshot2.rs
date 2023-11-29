use std::marker::PhantomData;

use bevy::prelude::*;
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

struct Builder<'w, I, C, R> {
    world: &'w World,
    entities: I,
    _marker: PhantomData<(C, R)>,
}

impl<'w> Builder<'w, std::array::IntoIter<Entity, 0>, (), ()> {
    pub fn new(world: &'w World) -> Self {
        Self {
            world,
            entities: [].into_iter(),
            _marker: PhantomData,
        }
    }
}

impl<'w, I, C, R> Builder<'w, I, C, R> {
    pub fn extract_component<T: Component + Serialize>(self) -> Builder<'w, I, (C, ExtComp<T>), R> {
        Builder {
            world: self.world,
            entities: self.entities,
            _marker: PhantomData,
        }
    }

    pub fn extract_resource<T: Resource + Serialize>(self) -> Builder<'w, I, C, (R, ExtRes<T>)> {
        Builder {
            world: self.world,
            entities: self.entities,
            _marker: PhantomData,
        }
    }
}

impl<'w, I, C, R> Builder<'w, I, C, R>
where
    I: Iterator<Item = Entity> + 'w,
{
    pub fn extract_entities<E>(
        self,
        entities: E,
    ) -> Builder<'w, impl Iterator<Item = Entity> + 'w, C, R>
    where
        E: Iterator<Item = Entity> + 'w,
    {
        Builder {
            world: self.world,
            entities: self.entities.chain(entities),
            _marker: PhantomData,
        }
    }

    pub fn extract_all_entities(self) -> Builder<'w, impl Iterator<Item = Entity> + 'w, C, R> {
        let entities = self.world.iter_entities().map(|e| e.id());
        self.extract_entities(entities)
    }
}

impl<'w, I, C, R> Builder<'w, I, C, R>
where
    I: Iterator<Item = Entity>,
    C: ExtractComponent<'w>,
    R: ExtractResource<'w>,
{
    pub fn build(self) -> Snapshot<'w, C, R> {
        Snapshot {
            entities: Entities(
                self.entities
                    .filter_map(|e| self.world.get_entity(e))
                    .map(|e| (e.id(), Extracted(C::extract(e))))
                    .collect(),
            ),
            resources: Extracted(R::extract(self.world)),
        }
    }
}

struct Snapshot<'w, C: ExtractComponent<'w>, R: ExtractResource<'w>> {
    entities: Entities<'w, C>,
    resources: Extracted<'w, R, &'w World>,
}

impl<'w, C, R> Serialize for Snapshot<'w, C, R>
where
    C: ExtractComponent<'w>,
    R: ExtractResource<'w>,
{
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

struct Entities<'w, C: ExtractComponent<'w>>(Vec<(Entity, Extracted<'w, C, EntityRef<'w>>)>);

impl<'w, C> Serialize for Entities<'w, C>
where
    C: ExtractComponent<'w>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ser = serializer.serialize_map(Some(self.0.len()))?;

        for (entity, components) in &self.0 {
            ser.serialize_key(entity)?;
            ser.serialize_value(components)?;
        }

        ser.end()
    }
}

impl<'a, 'de: 'a, C: 'a> Deserialize<'de> for Entities<'a, C>
where
    C: ExtractComponent<'a>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EntitiesVisitor<'a, C: 'a>(PhantomData<&'a C>);

        impl<'a, 'de: 'a, C: 'a> Visitor<'de> for EntitiesVisitor<'a, C>
        where
            C: ExtractComponent<'a> + 'a,
        {
            type Value = Entities<'a, C>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of extracted entities")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut entities = Vec::new();

                while let Some((key, value)) = map.next_entry()? {
                    entities.push((key, value));
                }

                Ok(Entities(entities))
            }
        }

        deserializer.deserialize_map(EntitiesVisitor(PhantomData))
    }
}

struct Extracted<'w, E: Extract<'w, I>, I: Copy + 'w>(E::Output);

impl<'w, E, I> Serialize for Extracted<'w, E, I>
where
    E: Extract<'w, I>,
    I: Copy + 'w,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ser = serializer.serialize_seq(None)?;
        E::serialize(&self.0, &mut ser)?;
        ser.end()
    }
}

impl<'de, E, I> Deserialize<'de> for Extracted<'de, E, I>
where
    E: Extract<'de, I>,
    I: Copy + 'de,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ExtractedVisitor<E, I>(PhantomData<(E, I)>);

        impl<'de, E, I> Visitor<'de> for ExtractedVisitor<E, I>
        where
            E: Extract<'de, I>,
            I: Copy + 'de,
        {
            type Value = Extracted<'de, E, I>;

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

impl<'de, C, R> Deserialize<'de> for Snapshot<'de, C, R>
where
    C: ExtractComponent<'de>,
    R: ExtractResource<'de>,
{
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

        impl<'de, C, R> Visitor<'de> for SnapshotVisitor<C, R>
        where
            C: ExtractComponent<'de>,
            R: ExtractResource<'de>,
        {
            type Value = Snapshot<'de, C, R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a snapshot struct")
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

#[test]
fn do_thing() {
    #[derive(Component, Serialize, Deserialize)]
    struct ExampleComponent {
        name: String,
    }

    #[derive(Component, Serialize, Deserialize)]
    struct OtherComponent;

    #[derive(Resource, Serialize, Deserialize)]
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

    let snapshot = Builder::new(world)
        .extract_component::<ExampleComponent>()
        .extract_component::<OtherComponent>()
        .extract_resource::<SimpleResource>()
        .extract_all_entities()
        .build();

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
            null
        ],
        "2": [
            null,
            null
        ],
        "3": [
            null,
            null
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

    let snapshot: Snapshot<
        '_,
        (((), ExtComp<ExampleComponent>), ExtComp<OtherComponent>),
        ((), ExtRes<SimpleResource>),
    > = Snapshot::deserialize(&mut de).unwrap();

    let mut buf = Vec::new();

    let format = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, format);

    snapshot.serialize(&mut ser).unwrap();

    let output = std::str::from_utf8(&buf).unwrap();

    assert_eq!(output, expected);
}

enum MaybeOwned<'a, T: 'a> {
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T> MaybeOwned<'a, T> {
    pub fn as_ref(&self) -> &T {
        match self {
            Self::Borrowed(val) => val,
            Self::Owned(val) => val,
        }
    }
}

impl<'a, T> From<&'a T> for MaybeOwned<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::Borrowed(value)
    }
}

impl<'a, T> std::ops::Deref for MaybeOwned<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, T: Serialize> Serialize for MaybeOwned<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

impl<'a, 'de: 'a, T: Deserialize<'de>> Deserialize<'de> for MaybeOwned<'a, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(MaybeOwned::Owned(T::deserialize(deserializer)?))
    }
}

struct ExtComp<C: Component + Serialize>(PhantomData<C>);
struct ExtRes<R: Resource + Serialize>(PhantomData<R>);

trait Extract<'w, I: Copy + 'w> {
    type Output: Serialize + Deserialize<'w>;

    fn extract(input: I) -> Self::Output;
    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error>;
    fn deserialize<'de: 'w, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error>;
}

impl<'w, T: Resource + Serialize + for<'de> Deserialize<'de>> Extract<'w, &'w World> for ExtRes<T> {
    type Output = Option<MaybeOwned<'w, T>>;

    fn extract(input: &'w World) -> Self::Output {
        input.get_resource::<T>().map(|o| o.into())
    }

    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(value)
    }

    fn deserialize<'de: 'w, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error> {
        seq.next_element()
    }
}

impl<'w, T: Component + Serialize + for<'de> Deserialize<'de>> Extract<'w, EntityRef<'w>>
    for ExtComp<T>
{
    type Output = Option<MaybeOwned<'w, T>>;

    fn extract(input: EntityRef<'w>) -> Self::Output {
        input.get::<T>().map(|o| o.into())
    }

    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(value)
    }

    fn deserialize<'de: 'w, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error> {
        seq.next_element()
    }
}

impl<'w, I: Copy + 'w> Extract<'w, I> for () {
    type Output = ();

    fn extract(_: I) -> Self::Output {}
    fn serialize<S: serde::ser::SerializeSeq>(_: &Self::Output, _: &mut S) -> Result<(), S::Error> {
        Ok(())
    }

    fn deserialize<'de, D: serde::de::SeqAccess<'de>>(_: &mut D) -> Result<Self::Output, D::Error> {
        Ok(())
    }
}

impl<'w, I: Copy + 'w, T0> Extract<'w, I> for (T0,)
where
    T0: Extract<'w, I>,
{
    type Output = (T0::Output,);

    fn extract(input: I) -> Self::Output {
        (T0::extract(input),)
    }

    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        T0::serialize(&value.0, seq)?;
        Ok(())
    }

    fn deserialize<'de: 'w, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error> {
        Ok((T0::deserialize(seq)?,))
    }
}

impl<'w, I: Copy + 'w, T0, T1> Extract<'w, I> for (T0, T1)
where
    T0: Extract<'w, I>,
    T1: Extract<'w, I>,
{
    type Output = (T0::Output, T1::Output);

    fn extract(input: I) -> Self::Output {
        (T0::extract(input), T1::extract(input))
    }

    fn serialize<S: serde::ser::SerializeSeq>(
        value: &Self::Output,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        T0::serialize(&value.0, seq)?;
        T1::serialize(&value.1, seq)?;
        Ok(())
    }

    fn deserialize<'de: 'w, D: serde::de::SeqAccess<'de>>(
        seq: &mut D,
    ) -> Result<Self::Output, D::Error> {
        Ok((T0::deserialize(seq)?, T1::deserialize(seq)?))
    }
}

trait ExtractComponent<'w>: Extract<'w, EntityRef<'w>> {}
impl<'w, T> ExtractComponent<'w> for T where T: Extract<'w, EntityRef<'w>> {}

trait ExtractResource<'w>: Extract<'w, &'w World> {}
impl<'w, T> ExtractResource<'w> for T where T: Extract<'w, &'w World> {}
