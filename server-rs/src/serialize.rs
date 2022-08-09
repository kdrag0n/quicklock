pub mod base64 {
    use serde::{Serialize, Deserialize};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = base64::encode(v);
        String::serialize(&base64, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(d)?;
        base64::decode(base64.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}

pub mod nested_b64 {
    use serde::{
        de::{DeserializeOwned, Deserializer, Error, Visitor},
        ser::{self, Serialize, Serializer},
    };
    use serde_json;
    use std::{fmt, marker::PhantomData};

    /// Deserialize value from a string which is valid JSON
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned,
    {
        #[derive(Default)]
        struct Helper<S: DeserializeOwned>(PhantomData<S>);

        impl<'de, S> Visitor<'de> for Helper<S>
        where
            S: DeserializeOwned,
        {
            type Value = S;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "valid json object")
            }

            fn visit_str<E>(self, value: &str) -> Result<S, E>
            where
                E: Error,
            {
                base64::decode(&value)
                    .map_err(Error::custom)
                    .and_then(|s| serde_json::from_slice(&s))
                    .map_err(Error::custom)
            }
        }

        deserializer.deserialize_str(Helper(PhantomData))
    }

    /// Serialize value as string containing JSON
    ///
    /// # Errors
    ///
    /// Serialization can fail if `T`'s implementation of `Serialize` decides to
    /// fail, or if `T` contains a map with non-string keys.
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        let s = serde_json::to_string(value).map_err(ser::Error::custom)?;
        let b64 = base64::encode(s.as_bytes());
        serializer.serialize_str(&b64)
    }
}
