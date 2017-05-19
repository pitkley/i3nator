// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use serde::de;
use serde::de::{Deserialize, Deserializer, DeserializeSeed};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

struct StringOrStruct<T>(PhantomData<T>);

impl<'de, T> de::Visitor<'de> for StringOrStruct<T>
    where T: Deserialize<'de> + FromStr<Err = String>
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("integer, string or map")
    }

    fn visit_i64<E>(self, value: i64) -> Result<T, E>
        where E: de::Error
    {
        FromStr::from_str(&value.to_string()).map_err(de::Error::custom)
    }

    fn visit_str<E>(self, value: &str) -> Result<T, E>
        where E: de::Error
    {
        FromStr::from_str(value).map_err(de::Error::custom)
    }

    fn visit_map<M>(self, visitor: M) -> Result<T, M::Error>
        where M: de::MapAccess<'de>
    {
        Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor))
    }
}

// Thanks to @dtolnay for the support:
//   https://github.com/serde-rs/serde/issues/901#issuecomment-297070279
impl<'de, T> DeserializeSeed<'de> for StringOrStruct<T>
    where T: Deserialize<'de> + FromStr<Err = String>
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_any(self)
    }
}

pub fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: Deserialize<'de> + FromStr<Err = String>,
          D: Deserializer<'de>
{
    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

pub fn option_string_or_struct<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where T: Deserialize<'de> + FromStr<Err = String>,
          D: Deserializer<'de>
{
    string_or_struct(deserializer).map(Some)
}

struct StringOrSeqString(PhantomData<Vec<String>>);

impl<'de> de::Visitor<'de> for StringOrSeqString {
    type Value = Vec<String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string or sequence of strings")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        Ok(vec![value.to_owned()])
    }

    fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where S: de::SeqAccess<'de>
    {
        Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))
    }
}

pub fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where D: Deserializer<'de>
{
    deserializer.deserialize_any(StringOrSeqString(PhantomData))
}

pub fn option_string_or_seq_string<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
    where D: Deserializer<'de>
{
    string_or_seq_string(deserializer).map(Some)
}

struct SingleOrSeqStringOrStruct<T>(PhantomData<T>);

impl<'de, T> de::Visitor<'de> for SingleOrSeqStringOrStruct<T>
    where T: Deserialize<'de> + FromStr<Err = String>
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("sequence of integers, strings or maps \
                             or a single integer, string or map")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where E: de::Error
    {
        FromStr::from_str(&value.to_string())
            .map(|e| vec![e])
            .map_err(de::Error::custom)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        FromStr::from_str(value)
            .map(|e| vec![e])
            .map_err(de::Error::custom)
    }

    fn visit_map<M>(self, visitor: M) -> Result<Self::Value, M::Error>
        where M: de::MapAccess<'de>
    {
        Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor)).map(|e| vec![e])
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where S: de::SeqAccess<'de>
    {
        let mut vec = Vec::new();
        while let Some(element) = seq.next_element_seed(StringOrStruct(PhantomData))? {
            vec.push(element);
        }
        Ok(vec)
    }
}

pub fn single_or_seq_string_or_struct<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where T: Deserialize<'de> + FromStr<Err = String>,
          D: Deserializer<'de>
{
    deserializer.deserialize_any(SingleOrSeqStringOrStruct(PhantomData))
}
