use std;
use std::fmt::{self, Display};

use Dynum::DynumError;
use serde::{de, ser};

use crate::SmithType;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    TryFromIntError,
    DynumError(DynumError),
    Message(String),
    SerdeMessage(String),
    Eof,
    TrailingCharacters,
    UTF8Error(std::str::Utf8Error),
    Expected(SmithType<usize>),
    UDIntNotCastInto(SmithType<usize>),
    EndOfStream,

    StrToCharError(&'static str),
    StringSerializerTypeNotString,

    Static(&'static str),
    CustomTypeNotFoundById,
    ExpectedArrayLen,
    DynamicNumberError(String),
    I64CastI32Failed,
    ValueNotHavingField(String),
    GivenStructureFieldAmoutNotMatching,
    ExpectedStructField(String),
    UnknownStruct(String),
    EnumVariantNotFound(String),
    MissmatchedType {
        expected: String,
        received: &'static str,
    },

    MissmatchedType2 {
        expected: String,
        received: SmithType<usize>,
    },
    
    EnumVariantWrongName {
        expected: String,
        received: String,
    },
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::SerdeMessage(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::SerdeMessage(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SerdeMessage(msg) => formatter.write_str(msg),
            _ => formatter.write_str(format!("{:?}", self).as_str()),
        }
    }
}

impl std::error::Error for Error {}
