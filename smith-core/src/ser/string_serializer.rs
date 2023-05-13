use std::panic::UnwindSafe;

use serde::{ser, Serialize};

use super::{Error, Result};

//Serializer that takes a generic value and serializes to a string
pub struct StringSerializer {
    pub val: Option<String>,
}

impl UnwindSafe for StringSerializer {}
impl StringSerializer {
    pub fn get<T: Serialize>(t: &T) -> Result<String> {
        let mut s = Self { val: None };

        t.serialize(&mut s)?;
        Ok(s.val.expect("Fatal Error! StringSerializer should always have an String after serializing the value"))
    }
}
//Only implemented method is serialize_str
impl<'a> ser::Serializer for &'a mut StringSerializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;

    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_str(self, v: &str) -> std::result::Result<Self::Ok, Self::Error> {
        self.val = Some(v.to_string());
        Ok(())
    }

    fn serialize_bool(self, _v: bool) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_i8(self, _v: i8) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_i16(self, _v: i16) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_i32(self, _v: i32) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_i64(self, _v: i64) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_u8(self, _v: u8) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_u16(self, _v: u16) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_u32(self, _v: u32) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_u64(self, _v: u64) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_f32(self, _v: f32) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_f64(self, _v: f64) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_char(self, _v: char) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_bytes(self, _v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_tuple(self, _len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }
}

impl<'a> ser::SerializeSeq for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::StringSerializerTypeNotString)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::StringSerializerTypeNotString)
    }

    fn end(self) -> Result<()> {
        Err(Error::StringSerializerTypeNotString)
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::StringSerializerTypeNotString)
    }

    fn end(self) -> Result<()> {
        Err(Error::StringSerializerTypeNotString)
    }
}

impl<'a> ser::SerializeStruct for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        Err(Error::StringSerializerTypeNotString)
    }

    fn end(self) -> Result<()> {
        Err(Error::StringSerializerTypeNotString)
    }
}

impl<'a> ser::SerializeMap for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        Err(Error::StringSerializerTypeNotString)
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        Err(Error::StringSerializerTypeNotString)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Err(Error::StringSerializerTypeNotString)
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::StringSerializerTypeNotString)
    }

    fn end(self) -> Result<()> {
        Err(Error::StringSerializerTypeNotString)
    }
}
