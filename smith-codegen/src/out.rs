#[doc = "this is a comment"]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Name {
    pub vor: String,
    pub nach: String,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Optional<T> {
    Some(T),
    None,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Person {
    pub name: Optional<Name>,
    pub alter: Optional<u32>,
}
