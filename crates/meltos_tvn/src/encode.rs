use auto_delegate::delegate;

use crate::error;

#[delegate]
pub trait Encodable {
    fn encode(&self) -> error::Result<Vec<u8>>;
}


pub trait Decodable: Sized {
    fn decode(buf: &[u8]) -> error::Result<Self>;
}


#[macro_export]
macro_rules! impl_serialize_and_deserialize {
    ($name: ident) => {
        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
                S::Error: serde::ser::Error,
            {
                let buf = &self.encode().unwrap();
                serializer.serialize_str(std::str::from_utf8(buf).unwrap())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visit;
                impl<'de> serde::de::Visitor<'de> for Visit {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(formatter, "format error")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name::decode(v.as_bytes()).unwrap())
                    }
                }
                deserializer.deserialize_str(Visit)
            }
        }
    };
}
