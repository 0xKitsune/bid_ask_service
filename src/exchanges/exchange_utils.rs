use std::fmt;

use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug)]
struct StringF64ArrayLen2Visitor;

//TODO: FIXME: We could use compile time constants,to make this one function, however serde derive does not support compile time constants at the moment
// So for now, we are using two different structs for array visitors with different lengths
impl<'a> Visitor<'a> for StringF64ArrayLen2Visitor {
    type Value = Vec<[f64; 2]>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a vector of two-element arrays of strings representing floats")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'a>,
    {
        let mut vec = vec![];

        while let Some(arr) = seq.next_element::<[String; 2]>()? {
            let first: f64 = arr[0].parse().map_err(de::Error::custom)?;
            let second: f64 = arr[1].parse().map_err(de::Error::custom)?;
            vec.push([first, second]);
        }

        Ok(vec)
    }
}

pub fn convert_array_len_2_to_f64<'a, D>(deserializer: D) -> Result<Vec<[f64; 2]>, D::Error>
where
    D: Deserializer<'a>,
{
    deserializer.deserialize_seq(StringF64ArrayLen2Visitor)
}

#[derive(Debug)]
struct StringF64ArrayLen3Visitor;
impl<'a> Visitor<'a> for StringF64ArrayLen3Visitor {
    type Value = Vec<[f64; 3]>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a vector of two-element arrays of strings representing floats")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'a>,
    {
        let mut vec = vec![];

        while let Some(arr) = seq.next_element::<[String; 3]>()? {
            let first: f64 = arr[0].parse().map_err(de::Error::custom)?;
            let second: f64 = arr[1].parse().map_err(de::Error::custom)?;
            let third: f64 = arr[3].parse().map_err(de::Error::custom)?;

            vec.push([first, second, third]);
        }

        Ok(vec)
    }
}

pub fn convert_array_len_3_to_f64<'a, D>(deserializer: D) -> Result<Vec<[f64; 3]>, D::Error>
where
    D: Deserializer<'a>,
{
    deserializer.deserialize_seq(StringF64ArrayLen3Visitor)
}

pub fn convert_from_string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
}
