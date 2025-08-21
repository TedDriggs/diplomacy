use std::{fmt, str::FromStr};

use std::ops::Deref;

use diplomacy::Command;
use diplomacy::geo::Location;
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SerializeDisplay, DeserializeFromStr,
)]
pub struct MapKey<O>(pub(crate) O);

impl<O> MapKey<O> {
    pub fn into_inner(self) -> O {
        self.0
    }
}

impl<O> Deref for MapKey<O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<O: fmt::Display> fmt::Display for MapKey<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<O: FromStr> FromStr for MapKey<O> {
    type Err = O::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl<O: Command<L>, L: Location> Command<L> for MapKey<O> {
    fn move_dest(&self) -> Option<&L> {
        self.0.move_dest()
    }
}

/// Provides serialization and deserialization for a map with keys that implement `Display` and `FromStr`.
/// This is useful for JSON serialization where keys need to be strings.
pub mod with_map_key {
    use std::{fmt::Display, marker::PhantomData, str::FromStr};

    use super::MapKey;
    use serde::{
        Deserialize, Serialize, Serializer,
        de::{Error, Visitor},
        ser::SerializeMap,
    };

    pub fn serialize<'a, S: Serializer, K: 'a + Serialize + Display, V: 'a + Serialize>(
        value: impl IntoIterator<Item = (&'a K, &'a V)>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_map(None)?;
        for (k, v) in value {
            s.serialize_entry(&MapKey(k), &v)?;
        }
        s.end()
    }

    pub fn deserialize<'de, D, K, V, C>(deserializer: D) -> Result<C, D::Error>
    where
        D: serde::Deserializer<'de>,
        K: Deserialize<'de> + FromStr,
        K::Err: Display,
        V: Deserialize<'de>,
        C: FromIterator<(K, V)>,
    {
        struct MapKeyVisitor<K, V, C>(PhantomData<(K, V, C)>);

        impl<K, V, C> Default for MapKeyVisitor<K, V, C> {
            fn default() -> Self {
                MapKeyVisitor(PhantomData)
            }
        }

        impl<'de, K, V, C> Visitor<'de> for MapKeyVisitor<K, V, C>
        where
            K: Deserialize<'de> + FromStr,
            K::Err: Display,
            V: Deserialize<'de>,
            C: FromIterator<(K, V)>,
        {
            type Value = C;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a map with keys that implement Display and FromStr")
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut entries: Vec<(K, V)> = vec![];
                while let Some((key, value)) = access.next_entry::<String, V>()? {
                    entries.push((FromStr::from_str(&key).map_err(A::Error::custom)?, value));
                }
                Ok(entries.into_iter().collect())
            }
        }

        deserializer.deserialize_map(MapKeyVisitor::default())
    }
}
