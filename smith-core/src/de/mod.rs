use serde::{Deserialize, forward_to_deserialize_any};
use serde::de::{
    self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};

use crate::generics_engine::SmithStruct;
use crate::resolver::ResolvedSmithProgram;
use crate::smith_serde::{Error, Result};
use crate::smith_serde::Error::StrToCharError;
use crate::SmithType;

type Data<'a> = &'a [u8];

struct BufferIter<'b, 'a> {
    buff: &'a mut &'b [u8],
}

impl<'b, 'a> Iterator for BufferIter<'b, 'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.buff.first() {
            let b = *t;
            *self.buff = &self.buff[1..];
            Some(b)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
enum EnumRepr {
    Json,
    Rust,
}

pub struct Deserializer<'de> {
    data: Data<'de>,
    prog: &'de ResolvedSmithProgram,
    current_type: &'de SmithType<usize>,
    enum_repr: EnumRepr,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(
        data: Data<'de>,
        prog: &'de ResolvedSmithProgram,
        current_type: &'de SmithType<usize>,
    ) -> Self {
        Self {
            data,
            prog,
            current_type,
            enum_repr: EnumRepr::Rust,
        }
    }

    pub fn from_bytes_json(
        data: Data<'de>,
        prog: &'de ResolvedSmithProgram,
        current_type: &'de SmithType<usize>,
    ) -> Self {
        Self {
            data,
            prog,
            current_type,
            enum_repr: EnumRepr::Json,
        }
    }

    #[inline(always)]
    pub fn read(&mut self) -> Result<u8> {
        if let Some(t) = self.data.first() {
            let b = *t;
            self.data = &self.data[1..];
            Ok(b)
        } else {
            Err(Error::EndOfStream)
        }
    }

    pub fn read_str(&mut self) -> Result<&'de str> {
        let mut i = 0;
        loop {
            if *self.data.get(i).ok_or(Error::EndOfStream)? == 0 {
                let s = std::str::from_utf8(&self.data[0..i]).map_err(|e| Error::UTF8Error(e));
                self.data = &self.data[i + 1..];
                return s;
            }
            i += 1;
        }
    }

    pub fn read_udint(&mut self) -> Result<u64> {
        let mut buffiter = BufferIter {
            buff: &mut self.data,
        };
        Dynum::decode_binary_stream(&mut buffiter).map_err(Error::DynumError)
    }

    pub fn read_n<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut buff = [0; N];
        for i in 0..N {
            buff[i] = self.read()?
        }
        Ok(buff)
    }
}

pub fn from_bytes<'a, T>(
    data: Data<'a>,
    prog: &'a ResolvedSmithProgram,
    current_type: &'a SmithType<usize>,
) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(data, prog, current_type);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.data.len() == 0 {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        bytes byte_buf option unit unit_struct newtype_struct tuple_struct
        tuple identifier ignored_any seq
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.current_type {
            SmithType::I8 => visitor.visit_i8(i8::from_be_bytes(self.read_n()?)),
            SmithType::I16 => visitor.visit_i16(i16::from_be_bytes(self.read_n()?)),
            SmithType::I32 => visitor.visit_i32(i32::from_be_bytes(self.read_n()?)),
            SmithType::F32 => visitor.visit_f32(f32::from_be_bytes(self.read_n()?)),
            SmithType::F64 => visitor.visit_f64(f64::from_be_bytes(self.read_n()?)),
            SmithType::U8 => visitor.visit_u8(u8::from_be_bytes(self.read_n()?)),
            SmithType::U16 => visitor.visit_u16(u16::from_be_bytes(self.read_n()?)),
            SmithType::U32 => visitor.visit_u32(u32::from_be_bytes(self.read_n()?)),
            SmithType::U64 => visitor.visit_u64(u64::from_be_bytes(self.read_n()?)),
            SmithType::UInt => visitor.visit_u64(self.read_udint()?),
            SmithType::Bool => visitor.visit_bool(u8::from_be(self.read()?) != 0),
            SmithType::String => self.deserialize_str(visitor),
            SmithType::CustomType(id, _) => {
                let t = self.prog.get(*id).ok_or(Error::CustomTypeNotFoundById)?;
                match &t.variant {
                    crate::generics_engine::SmithCustomTypVariant::Struct(_) => {
                        return self.deserialize_map(visitor);
                    }
                    crate::generics_engine::SmithCustomTypVariant::Enum(_e) => {
                        return self.deserialize_enum("", &[], visitor);
                    }
                }
            }
            SmithType::Array(t) => {
                let len = self.read_udint()?;
                visitor.visit_seq(SeqVisitor {
                    de: self,
                    eltyp: t,
                    remaining: len as usize,
                })
            }
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let SmithType::String = self.current_type {
            let mut chars = self.read_str()?.chars();
            let only_char = chars
                .next()
                .ok_or(Error::StrToCharError("String is empty - no char to"))?;
            if chars.next().is_some() {
                return Err(StrToCharError("Stringlen > 1"));
            }
            visitor.visit_char(only_char)
        } else {
            Err(Error::Expected(SmithType::String))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let SmithType::String = self.current_type {
            visitor.visit_borrowed_str(self.read_str()?)
        } else {
            Err(Error::Expected(SmithType::String))
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let SmithType::String = self.current_type {
            visitor.visit_str(self.read_str()?)
        } else {
            Err(Error::Expected(SmithType::String))
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let SmithType::CustomType(id, _) = self.current_type {
            let s = self
                .prog
                .get(*id)
                .ok_or(Error::CustomTypeNotFoundById)?
                .as_struct()
                .ok_or(Error::MissmatchedType {
                    expected: "struct".to_owned(),
                    received: "enum",
                })?;
            return visitor.visit_map(StructVisitor {
                de: self,
                structyp: s,
                current_field_idx: 0,
            });
        } else {
            return Err(Error::MissmatchedType2 {
                expected: "struct".to_owned(),
                received: self.current_type.clone(),
            });
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let SmithType::CustomType(id, _) = self.current_type {
            let s = self
                .prog
                .get(*id)
                .ok_or(Error::CustomTypeNotFoundById)?
                .as_enum()
                .ok_or(Error::MissmatchedType {
                    expected: "struct".to_owned(),
                    received: "enum",
                })?;

            let varid = self.read_udint()?;
            let variant = &s.variants[varid as usize];

            if _name.len() > 0 && _variants[varid as usize] != variant.0 {
                return Err(Error::EnumVariantWrongName {
                    expected: _variants[varid as usize].to_owned(),
                    received: variant.0.to_owned(),
                });
            }
            let repr = self.enum_repr;
            let evisit = EnumVisitor { de: self, variant };
            match repr {
                EnumRepr::Json => visitor.visit_map(EnumAsMapVisitor(evisit, 0)),
                EnumRepr::Rust => visitor.visit_enum(evisit),
            }
        } else {
            return Err(Error::MissmatchedType {
                expected: "struct".to_owned(),
                received: "else",
            });
        }
    }
}

struct StructVisitor<'a, 'de: 'a> {
    pub de: &'a mut Deserializer<'de>,
    pub structyp: &'de SmithStruct<usize>,
    pub current_field_idx: usize,
}

impl<'de, 'a> MapAccess<'de> for StructVisitor<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some(s) = self.structyp.fields.get(self.current_field_idx) {
            self.de.current_type = &s.1;
            self.current_field_idx += 1;
            seed.deserialize(&mut StringDeserializer { s: s.0.as_str() })
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct SeqVisitor<'a, 'de: 'a> {
    pub de: &'a mut Deserializer<'de>,
    pub eltyp: &'de SmithType<usize>,
    pub remaining: usize,
}

impl<'de, 'a> SeqAccess<'de> for SeqVisitor<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        self.de.current_type = self.eltyp;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct EnumVisitor<'a, 'de: 'a> {
    pub de: &'a mut Deserializer<'de>,
    pub variant: &'de (String, Option<SmithType<usize>>),
}

impl<'de, 'a> EnumAccess<'de> for EnumVisitor<'a, 'de> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> std::result::Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((
            seed.deserialize(&mut StringDeserializer {
                s: self.variant.0.as_str(),
            })?,
            self,
        ))
    }
}

struct EnumAsMapVisitor<'a, 'de: 'a>(EnumVisitor<'a, 'de>, u8);

impl<'de, 'a> MapAccess<'de> for EnumAsMapVisitor<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let i = self.1;
        if i == 0 {
            return Ok(Some(
                seed.deserialize(&mut StringDeserializer { s: "tag" })?,
            ));
        } else if i == 1 {
            return if let Some(_s) = &self.0.variant.1 {
                Ok(Some(
                    seed.deserialize(&mut StringDeserializer { s: "val" })?,
                ))
            } else {
                Ok(None)
            };
        } else {
            return Ok(None);
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let i = self.1;
        self.1 += 1;
        if i == 0 {
            Ok(seed.deserialize(&mut StringDeserializer {
                s: self.0.variant.0.as_str(),
            })?)
        } else if i == 1 {
            if let Some(s) = &self.0.variant.1 {
                self.0.de.current_type = s;
                seed.deserialize(&mut *self.0.de)
            } else {
                seed.deserialize(&mut NoneDeserializer)
            }
        } else {
            panic!("{:?}", self.1);
        }
    }
}

impl<'de, 'a> VariantAccess<'de> for EnumVisitor<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> std::result::Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.de.current_type = self
            .variant
            .1
            .as_ref()
            .ok_or(Error::Message(format!("enumvariant {} is not newtype", self.variant.0)))?;
        seed.deserialize(&mut *self.de)
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("Smith does not support tuple variants for enums")
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("Smith does not support struct variants for enums")
    }
}

///
/// Only purpose is to encapsulate a string
///
struct StringDeserializer<'a> {
    s: &'a str,
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut StringDeserializer<'de> {
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.s)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct NoneDeserializer;

impl<'de, 'a> de::Deserializer<'de> for &'a mut NoneDeserializer {
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_none()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
