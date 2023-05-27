use std::fmt;

use serde::{
    de::{self, SeqAccess, Visitor},
    Deserializer,
};

#[derive(Debug)]
struct StringF64ArrayVisitor;

impl<'a> Visitor<'a> for StringF64ArrayVisitor {
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

pub fn convert_array_items_to_f64<'a, D>(deserializer: D) -> Result<Vec<[f64; 2]>, D::Error>
where
    D: Deserializer<'a>,
{
    deserializer.deserialize_seq(StringF64ArrayVisitor)
}
