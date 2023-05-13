use serde::{ser, Serialize};
use serde::ser::SerializeStruct;

use serialize_struct::{EnumSerializer, StructSerializer};
use smith_serde::{Error, Result};

use crate::{
    smith_serde, SmithType,
};
use crate::ser::serialize_struct::StructEnumSerializer;

use super::resolver::ResolvedSmithProgram;

mod serialize_struct;
mod string_serializer;

macro_rules! require_type {
    ($Path:path, $Self:ident) => {{
        if let $Path = $Self.current_type {
        } else {
            return Err(Error::MissmatchedType {
                expected: format!("{:?}", $Self.current_type),
                received: stringify!($Path),
            });
        };
    }};
}

macro_rules! serialize_number {
    ($Self:ident, $V:ident) => {
        match $Self.current_type {
            SmithType::I8 => $Self.buff.extend_from_slice(
                &i8::try_from($V)
                    .map_err(|_| Error::TryFromIntError)?
                    .to_be_bytes(),
            ),
            SmithType::I16 => $Self.buff.extend_from_slice(
                &i16::try_from($V)
                    .map_err(|_| Error::TryFromIntError)?
                    .to_be_bytes(),
            ),
            SmithType::I32 => $Self.buff.extend_from_slice(
                &i32::try_from($V)
                    .map_err(|_| Error::TryFromIntError)?
                    .to_be_bytes(),
            ),
            SmithType::F32 => $Self.buff.extend_from_slice(&($V as f32).to_be_bytes()),
            SmithType::F64 => $Self.buff.extend_from_slice(&($V as f64).to_be_bytes()),
            SmithType::U8 => $Self.buff.extend_from_slice(
                &u8::try_from($V)
                    .map_err(|_| Error::TryFromIntError)?
                    .to_be_bytes(),
            ),
            SmithType::U16 => $Self.buff.extend_from_slice(
                &u16::try_from($V)
                    .map_err(|_| Error::TryFromIntError)?
                    .to_be_bytes(),
            ),
            SmithType::U32 => $Self.buff.extend_from_slice(
                &u32::try_from($V)
                    .map_err(|_| Error::TryFromIntError)?
                    .to_be_bytes(),
            ),
            SmithType::U64 => $Self.buff.extend_from_slice(
                &u64::try_from($V)
                    .map_err(|_| Error::TryFromIntError)?
                    .to_be_bytes(),
            ),
            SmithType::UInt => {
                Dynum::encode_into(
                    u64::try_from($V).map_err(|_| Error::TryFromIntError)?,
                    |v| $Self.buff.push(v),
                )
                .map_err(Error::DynumError)?;
            }
            _ => {
                panic!("Expected number")
            }
        };
    };
}

pub struct Serializer<'a> {
    pub buff: Vec<u8>,
    prog: &'a ResolvedSmithProgram,
    current_type: &'a SmithType<usize>,
}

impl<'a> Serializer<'a> {
    pub fn new(prog: &'a ResolvedSmithProgram, typ: &'a SmithType<usize>) -> Self {
        Self {
            buff: Vec::with_capacity(64),
            prog,
            current_type: typ,
        }
    }

    pub fn buffer(self) -> Vec<u8> {
        self.buff
    }
}

pub fn to_binary<T>(
    value: &T,
    prog: &ResolvedSmithProgram,
    typ: &SmithType<usize>,
) -> Result<Box<[u8]>>
    where
        T: Serialize,
{
    let mut serializer = Serializer::new(prog, typ);
    value.serialize(&mut serializer)?;
    Ok(serializer.buff.into_boxed_slice())
}

impl<'a, 'b> ser::Serializer for &'a mut Serializer<'b> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = SeqSerializer<'a, 'b>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;

    type SerializeStruct = StructEnumSerializer<'a, 'b>;
    type SerializeMap = StructEnumSerializer<'a, 'b>;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        if let SmithType::Bool = self.current_type {
            self.buff.push((v as u8).to_be());
            Ok(())
        } else {
            Err(Error::MissmatchedType {
                expected: format!("{:?}", self.current_type),
                received: "bool",
            })
        }
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        serialize_number!(self, v);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        serialize_number!(self, v);
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        serialize_number!(self, v);
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_i32(v.try_into().map_err(|_| Error::I64CastI32Failed)?)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        serialize_number!(self, v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        serialize_number!(self, v);
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        serialize_number!(self, v);
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        serialize_number!(self, v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        match self.current_type {
            SmithType::F32 => self.buff.extend_from_slice(&v.to_be_bytes()),
            SmithType::F64 => self.buff.extend_from_slice(&(v as f64).to_be_bytes()),
            _ => {
                return Err(Error::MissmatchedType {
                    expected: "Float".to_string(),
                    received: "something else",
                });
            }
        };
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        match self.current_type {
            SmithType::F32 => self.buff.extend_from_slice(&(v as f32).to_be_bytes()),
            SmithType::F64 => self.buff.extend_from_slice(&v.to_be_bytes()),
            _ => {
                return Err(Error::MissmatchedType {
                    expected: "Float".to_string(),
                    received: "something else",
                });
            }
        };
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        require_type!(SmithType::String, self);
        self.buff.extend_from_slice(v.as_bytes());
        self.buff.push(0);
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit(self) -> Result<()> {
        unimplemented!("SMITH does not support serialization of empty unit data")
    }

    /*
    ================
    SECTION ENUMS
    ================
     */

    /// enum Example{
    ///     A, <- unit variant
    ///     B, <- unit variant
    ///     C(...) <- not unit variant
    /// }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_newtype_variant(_name, _variant_index, variant, &Option::<u8>::None)
    }

    /// enum Example{
    ///     A, <- unit variant
    ///     C(SomeType) <- newtype variant
    /// }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        if let SmithType::CustomType(id, _) = self.current_type {
            let s = self
                .prog
                .get(*id)
                .ok_or(Error::CustomTypeNotFoundById)?
                .as_enum()
                .ok_or(Error::MissmatchedType {
                    expected: "enum".to_owned(),
                    received: "struct",
                })?;

            let variantdata = s
                .variants
                .get(_variant_index as usize)
                .ok_or(Error::Static("Enum variants count not matching"))?;
            if variant != variantdata.0 {
                println!("WARNING: for enum '{_name}' - provided variant '{variant}' name does not match with '{}' at index {_variant_index}", variantdata.0)
            }
            self.current_type = &SmithType::UInt;
            _variant_index.serialize(&mut *self)?;
            if let Some(s) = &variantdata.1 {
                self.current_type = s;
                value.serialize(&mut *self)?;
            }
            return Ok(());
        } else {
            return Err(Error::MissmatchedType {
                expected: format!("{:?}", self.current_type),
                received: "enum",
            });
        }
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!("SMITH Does not support struct variants")
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!("Smith does not support enum tuple variants!")
    }

    /*
    ================
    SECTION ARRAYS
    ================
     */

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        if let SmithType::Array(a) = self.current_type {
            if let Some(_len) = _len {
                self.current_type = &SmithType::UInt;
                _len.serialize(&mut *self)?
            } else {
                return Err(Error::ExpectedArrayLen);
            }

            Ok(SeqSerializer {
                elemtyp: a,
                serializer: self,
            })
        } else {
            Err(Error::MissmatchedType {
                expected: format!("{:?}", self.current_type),
                received: "Array",
            })
        }
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        unimplemented!()
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unimplemented!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        println!("WARNING Serializing bytes directly is highly dangerous!");
        self.buff.extend_from_slice(v);
        Ok(())
    }

    /*
    ================
    SECTION STRUCTS
    ================
     */
    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.serialize_struct("", 0)
    }

    /// Ignores the name of given struct - only name of schema matters
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct> {
        if let SmithType::CustomType(id, _gen) = self.current_type {
            let typ = self.prog.get(*id).ok_or(Error::CustomTypeNotFoundById)?;
            match &typ.variant {
                crate::generics_engine::SmithCustomTypVariant::Struct(s) => {
                    return Ok(StructEnumSerializer::newstruct(StructSerializer {
                        structyp: s,
                        serializer: self,
                        current_field_idx: 0,
                    }));
                }
                crate::generics_engine::SmithCustomTypVariant::Enum(e) => {
                    return Ok(StructEnumSerializer::newenum(EnumSerializer {
                        serializer: self,
                        smith_enum: e,
                        enum_variant_type: None,
                    }));
                }
            }
        } else {
            return Err(Error::MissmatchedType {
                expected: format!("{:?}", self.current_type),
                received: "struct",
            });
        }
    }
    /// EXAMPLE: struct Test;
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_struct(_name, 0)?;
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        unimplemented!("SMITH does not support serialization of newtype structs")
    }
}

pub struct SeqSerializer<'a, 'b> {
    pub elemtyp: &'b SmithType<usize>,
    pub serializer: &'a mut Serializer<'b>,
}

impl<'a, 'b> ser::SerializeSeq for SeqSerializer<'a, 'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        self.serializer.current_type = self.elemtyp;
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'b> ser::SerializeTuple for &'a mut Serializer<'b> {
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

impl<'a, 'b> ser::SerializeTupleStruct for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        unimplemented!("Tuple structs not supported")
    }

    fn end(self) -> Result<()> {
        unimplemented!("Tuple structs not supported")
    }
}

impl<'a, 'b> ser::SerializeTupleVariant for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        unimplemented!("enums variants with tuple fields not supported")
    }

    fn end(self) -> Result<()> {
        unimplemented!("enums variants with tuple fields not supported")
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a, 'b> ser::SerializeStructVariant for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        unimplemented!("enums with variant structs not supported")
    }

    fn end(self) -> Result<()> {
        unimplemented!("enums with variant structs not supported")
    }
}
