use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};
use smith_core::ser::AsyncSerializer;
use std::{
    ops::{Deref, DerefMut},
    todo,
};

#[derive(Debug, PartialEq, Clone)]
pub struct AsyncVec<T>(Vec<T>);
impl<T> AsyncVec<T> {
    pub fn into_inner(self) -> Vec<T> {
        return self.0;
    }
}

trait Ser<S: serde::Serializer> {
    fn serialize_maybeasync<B: Serialize + Sync>(self, val: &Vec<B>) -> Result<S::Ok, S::Error>;
}

//Default implemenentation
impl<T: serde::Serializer> Ser<T> for T {
    default fn serialize_maybeasync<B: Serialize + Sync>(
        self,
        val: &Vec<B>,
    ) -> Result<T::Ok, T::Error> {
        let mut seqser = self.serialize_seq(Some(val.len()))?;

        let res = val.iter().map(|e| seqser.serialize_element(e));

        for r in res {
            r?
        }

        seqser.end()
    }
}

//If Serializer is of type AsyncSerializer, use it
impl<'a, T: AsyncSerializer> Ser<T> for T {
    fn serialize_maybeasync<B: Serialize + Sync>(self, val: &Vec<B>) -> Result<T::Ok, T::Error> {
        self.serialize_slice_async(&val)
    }
}

impl<T: Serialize + Sync> Serialize for AsyncVec<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_maybeasync(&self.0)
    }
}

impl<'a, T: Deserialize<'a> + Serialize + Sync> Deserialize<'a> for AsyncVec<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        Ok(AsyncVec(Vec::deserialize(deserializer)?))
    }
}

impl<'a, T: Serialize + Sync> Deref for AsyncVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<'a, T: Serialize + Sync> DerefMut for AsyncVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
