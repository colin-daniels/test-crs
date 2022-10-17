use super::SourceType;
use crate::engine::RequestExt;
use http::Request;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ValueData<'a> {
    Named { name: &'a [u8], value: &'a [u8] },
    Value(&'a [u8]),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Value<'a> {
    source: SourceType,
    data: ValueData<'a>,
}

impl<'a> Value<'a> {
    #[inline]
    pub fn new(source: SourceType, value: &'a [u8]) -> Self {
        Self {
            source,
            data: ValueData::Value(value),
        }
    }

    #[inline]
    pub fn new_named(source: SourceType, name: &'a [u8], value: &'a [u8]) -> Self {
        Self {
            source,
            data: ValueData::Named { name, value },
        }
    }

    #[inline]
    pub fn from_str(source: SourceType, value: &'a str) -> Self {
        Self::new(source, value.as_bytes())
    }

    #[inline]
    pub fn from_str_named(source: SourceType, name: &'a str, value: &'a str) -> Self {
        Self::new_named(source, name.as_bytes(), value.as_bytes())
    }

    #[inline]
    pub fn source(&self) -> SourceType {
        self.source
    }

    #[inline]
    pub fn value(&self) -> &'a [u8] {
        match self.data {
            ValueData::Value(value) => value,
            ValueData::Named { value, .. } => value,
        }
    }

    #[inline]
    pub fn name(&self) -> Option<&'a [u8]> {
        match self.data {
            ValueData::Value(_) => None,
            ValueData::Named { name, .. } => Some(name),
        }
    }

    #[inline]
    pub fn into_name(self, source: SourceType) -> Option<Value<'a>> {
        match self.data {
            ValueData::Value(_) => None,
            ValueData::Named { name, .. } => Some(Value::new(source, name)),
        }
    }
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_tuple(self.source.name());
        match self.data {
            ValueData::Named { name, value } => debug
                .field(&String::from_utf8_lossy(name))
                .field(&String::from_utf8_lossy(value))
                .finish(),
            ValueData::Value(value) => debug.field(&String::from_utf8_lossy(value)).finish(),
        }
    }
}
