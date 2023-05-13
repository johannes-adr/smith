use serde::{ser, Serialize};

use crate::generics_engine::{SmithEnum, SmithStruct};
use crate::ser::{Serializer, string_serializer};
use crate::SmithType;

use super::{Error, Result};

pub struct StructEnumSerializer<'a, 'b> {
    curr_key: Option<String>,
    variants: StructEnumSerializerVariants<'a, 'b>,
}

enum StructEnumSerializerVariants<'a, 'b> {
    Struct(StructSerializer<'a, 'b>),
    Enum(EnumSerializer<'a, 'b>),
}

impl<'a, 'b> StructEnumSerializer<'a, 'b> {
    #[inline(always)]
    fn serialize_field_strkey<T: ?Sized>(&mut self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        match &mut self.variants {
            StructEnumSerializerVariants::Struct(s) => s.serialize_field(key, value),
            StructEnumSerializerVariants::Enum(e) => e.serialize_enum(key, value),
        }
    }

    pub fn newstruct(s: StructSerializer<'a, 'b>) -> Self {
        Self {
            curr_key: None,
            variants: StructEnumSerializerVariants::Struct(s),
        }
    }

    pub fn newenum(e: EnumSerializer<'a, 'b>) -> Self {
        Self {
            curr_key: None,
            variants: StructEnumSerializerVariants::Enum(e),
        }
    }
}

impl<'a, 'b> ser::SerializeStruct for StructEnumSerializer<'a, 'b> {
    type Ok = ();
    type Error = Error;
    #[inline(always)]
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_field_strkey(key, value)
    }
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'b> ser::SerializeMap for StructEnumSerializer<'a, 'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        let key = string_serializer::StringSerializer::get(&key)?;
        self.curr_key = Some(key);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        if let Some(s) = self.curr_key.take() {
            return self.serialize_field_strkey(&s, value);
        }
        Err(Error::Message(
            "StructEnumSerializer tried to serialize value without key supplied".to_string(),
        ))
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> std::result::Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        let key = string_serializer::StringSerializer::get(&key)?;
        self.serialize_field_strkey(&key, value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

pub struct EnumSerializer<'a, 'b> {
    pub smith_enum: &'b SmithEnum<usize>,
    pub enum_variant_type: Option<&'b (String, Option<SmithType<usize>>)>,
    pub serializer: &'a mut Serializer<'b>,
}

impl<'a, 'b> EnumSerializer<'a, 'b> {
    fn serialize_enum<T: ?Sized>(&mut self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        return if key == "tag" {
            let name = string_serializer::StringSerializer::get(&value)?;
            let variantpos = self
                .smith_enum
                .variants
                .iter()
                .position(|e| e.0 == name)
                .ok_or_else(|| Error::EnumVariantNotFound(name.to_string()))?;
            let variant = &self.smith_enum.variants[variantpos];

            self.serializer.current_type = &SmithType::UInt;
            variantpos.serialize(&mut *self.serializer)?;

            self.enum_variant_type = Some(variant);
            Ok(())
        } else if key == "val" {
            if let Some(s) = &self.enum_variant_type {
                if let Some(t) = &s.1 {
                    self.serializer.current_type = t;
                    value.serialize(&mut *self.serializer)
                } else {
                    Err(Error::Message(format!(
                        "SmithEnum '{}' with variant '{}' does not have a value",
                        &self.smith_enum.name, s.0
                    )))
                }
            } else {
                Err(Error::Message(format!(
                    "SmithEnum '{}': variant tag not read yet",
                    &self.smith_enum.name
                )))
            }
        } else {
            Err(Error::Message(format!(
                "SmithEnum '{}' expected, only valid map keys are 'tag' and 'val' - '{}' provided",
                self.smith_enum.name, key
            )))
        };
    }
}

pub struct StructSerializer<'a, 'b> {
    pub structyp: &'b SmithStruct<usize>,
    pub serializer: &'a mut Serializer<'b>,
    pub current_field_idx: usize,
}

impl<'a, 'b> StructSerializer<'a, 'b> {
    fn serialize_field<T>(&mut self, key: &str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let f = self
            .structyp
            .fields
            .get(self.current_field_idx)
            .ok_or(Error::GivenStructureFieldAmoutNotMatching)?;
        if f.0 != key {
            return Err(Error::ExpectedStructField(format!(
                "expected: '{}' - got '{}'",
                f.0, key
            )));
        }
        let t = &f.1;
        self.serializer.current_type = t;
        self.current_field_idx += 1;
        value.serialize(&mut *self.serializer)
    }
}
