use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Packet {
    pub id: u8,
    pub payload: PacketType,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Order<T> {
    pub table_number: u64,
    pub items: Optional<Box<[T]>>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OrderItem {
    pub id: u8,
    pub amount: u8,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Person {
    pub name: String,
    pub age: u8,
    pub desc: Optional<String>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum PacketType {
    Ack,
    LogOut,
    Order(Order<OrderItem>),
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Optional<T> {
    Some(T),
    None,
}
