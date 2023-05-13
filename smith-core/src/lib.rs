#[macro_use]
extern crate pest_derive;

use std::borrow::Borrow;
use std::sync::Arc;

use generics_engine::SmithCustomTyp;
use serde::{Deserialize, Serialize};

pub use generics_engine::SmithProgram;
pub use parser::SmithType;

use crate::resolver::ResolvedSmithProgram;
use crate::smith_serde::Error;

pub mod generics_engine;
pub mod parser;
pub mod resolver;
mod utils;

pub mod de;
pub mod ser;
mod smith_serde;

pub fn compile(src: &str) -> SmithProgram<usize> {
    let parsed = parser::parse(src);
    let expanded = generics_engine::expand(parsed).unwrap();
    resolver::resolve(expanded).unwrap()
}

#[derive(Clone)]
pub struct Smith(Arc<ResolvedSmithProgram>);

impl Smith {
    pub fn get_type(&self, name: &str) -> Option<SmithType<usize>> {
        self.0
            .iter()
            .position(|e| e.get_name() == name)
            .map(|e| SmithType::CustomType(e, vec![]))
    }

    pub fn get_types(&self) -> &[SmithCustomTyp<usize>] {
        (*self.0).as_slice()
    }

    pub fn new(src: &str) -> Self {
        Self(Arc::new(compile(src).expanded))
    }

    pub fn rust2binary<T>(&self, value: &T, typ: &SmithType<usize>) -> Result<Box<[u8]>, Error>
    where
        T: Serialize,
    {
        ser::to_binary(value, &self.0, typ)
    }

    pub fn binary2rust<'a, T>(
        &'a self,
        data: &'a [u8],
        typ: &'a SmithType<usize>,
    ) -> Result<T, Error>
    where
        T: Deserialize<'a>,
    {
        de::from_bytes(data, &self.0, typ)
    }
}

impl Smith {
    pub fn json2binary(&self, json: &str, typ: &SmithType<usize>) -> Result<Box<[u8]>, String> {
        let mut ser = ser::Serializer::new(&self.0, typ);
        serde_transcode::transcode(
            &mut json5::Deserializer::from_str(json).map_err(|e| e.to_string())?,
            &mut ser,
        )
        .map_err(|e| format!("{e:?}"))?;

        Ok(ser.buffer().into_boxed_slice())
    }
    pub fn binary2json(&self, bin: &[u8], typ: &SmithType<usize>) -> Result<String, String> {
        let mut buf = Vec::new();
        serde_transcode::transcode(
            &mut de::Deserializer::from_bytes_json(&bin, &self.0, typ),
            &mut serde_json::Serializer::new(&mut buf),
        )
        .map_err(|e| format!("{e:?}"))?;
        Ok(String::from_utf8(buf).map_err(|e| e.to_string())?)
    }
}

/*
=============
TESTS
=============
*/

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_schemaread() {
        let s = Smith::new(SCHEMA);
        let required_types = [
            "OrderItem",
            "Person",
            "Order<OrderItem>",
            "Optional<Array<OrderItem>>",
            "Optional<string>",
            "Packet",
            "PacketType",
        ];
        for name in required_types {
            let typ = s.get_type(name);
            assert!(typ.is_some(), "failed for {}", name)
        }
        assert_eq!(s.get_types().len(), required_types.len());
    }

    #[test]
    fn test_json2bin() {
        let s = Smith::new(SCHEMA);
        let typ = s.get_type("Packet").unwrap();
        let res = s.json2binary(&SRC_JSON, &typ).unwrap();
        assert_eq!(res, BIN.to_vec().into_boxed_slice());
    }

    #[test]
    fn test_bin2json() {
        let s = Smith::new(SCHEMA);
        let typ = s.get_type("Packet").unwrap();
        let res = s.binary2json(BIN, &typ).unwrap();
        assert_eq!(res, SRC_JSON);
    }

    #[test]
    fn test_bin2rust() {
        let s = Smith::new(SCHEMA);
        let typ = s.get_type("Packet").unwrap();
        let res: Types::Packet = s.binary2rust(BIN, &typ).unwrap();
        assert_eq!(res, Types::get_rustvalue());
    }

    #[test]
    fn test_rust2bin() {
        let s = Smith::new(SCHEMA);
        let typ = s.get_type("Packet").unwrap();
        let res = s.rust2binary(&Types::get_rustvalue(), &typ).unwrap();
        assert_eq!(res, BIN.to_vec().into_boxed_slice());
    }




    /*
    ================
    STATIC RESOURCES
    ================
    */

    const SRC_JSON: &str = r#"{"id":1,"payload":{"tag":"Order","val":{"table_number":1,"items":{"tag":"Some","val":[{"id":1,"amount":2},{"id":2,"amount":2},{"id":3,"amount":3},{"id":4,"amount":4}]}}}}"#;
    const BIN: &[u8] = &[1, 5, 3, 1, 9, 1, 2, 2, 2, 3, 3, 4, 4];
    const SCHEMA: &str = r#"
    enum Optional<T>{
        Some(T)
        None
    }
    
    struct Packet{
        id: u8
        payload: PacketType
    }
    
    //Dont work - Array<T> will not generate a optional with string type
    struct Order<T>{
        table_number: udInt
        items: Optional<Array<T>>
    }
    
    enum PacketType{
        Ack
        LogOut
        Order(Order<OrderItem>)
    }

    struct OrderItem{
        id: u8
        amount: u8
    }
    
    struct Person{
        name: string
        age: u8
        desc: Optional<string>
    }"#;

    mod Types {
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

        pub fn get_rustvalue() -> Packet {
            Packet {
                id: 1,
                payload: PacketType::Order(Order {
                    table_number: 1,
                    items: Optional::Some(
                        vec![
                            OrderItem { id: 1, amount: 2 },
                            OrderItem { id: 2, amount: 2 },
                            OrderItem { id: 3, amount: 3 },
                            OrderItem { id: 4, amount: 4 },
                        ]
                        .into_boxed_slice(),
                    ),
                }),
            }
        }
    }
}
