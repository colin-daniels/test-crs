use serde::de::{EnumAccess, Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;

pub fn for_each<'de, D, T, F>(deserializer: D, f: F) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
    F: FnMut(T),
{
    struct StrVisitor<T, F>(F, PhantomData<T>);

    impl<'de, T, F> Visitor<'de> for StrVisitor<T, F>
    where
        T: Deserialize<'de>,
        F: FnMut(T),
    {
        type Value = ();

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            todo!()
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!()
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            todo!()
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            todo!("visit_unit")
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            todo!("visit_newtype_struct")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            todo!("visit_seq")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            todo!("visit_map")
        }

        fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
        where
            A: EnumAccess<'de>,
        {
            todo!("enum")
        }
    }

    let visitor = StrVisitor(f, PhantomData);
    deserializer.deserialize_any(visitor)
}
