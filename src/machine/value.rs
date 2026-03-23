use super::buffer::*;
use super::error::*;
use crate::parse::construct::*;

//================================================================

use std::fmt::Debug;

//================================================================

pub enum Value {
    Null,
    String(String),
    Integer(i32),
    Decimal(f32),
    Boolean(bool),
    Function(Function),
    FunctionNative(Box<dyn Fn(ArgumentBuffer) -> Result<Value, Error>>),
    Enumerate(Enumerate),
    Structure(Structure),
}

impl Debug for Value {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null              => f.debug_tuple("Null").finish(),
            Self::String(value)     => f.debug_tuple("String").field(value).finish(),
            Self::Integer(value)    => f.debug_tuple("Integer").field(value).finish(),
            Self::Decimal(value)    => f.debug_tuple("Decimal").field(value).finish(),
            Self::Boolean(value)    => f.debug_tuple("Boolean").field(value).finish(),
            Self::Function(value)   => f.debug_tuple("Function").field(value).finish(),
            Self::FunctionNative(_) => f.debug_tuple("FunctionNative").finish(),
            Self::Enumerate(value)  => f.debug_tuple("Enumerate").field(value).finish(),
            Self::Structure(value)  => f.debug_tuple("Structure").field(value).finish(),
        }
    }
}

impl Into<String> for Value {
    #[rustfmt::skip]
    fn into(self) -> String {
        let string = match self {
            Self::Null              => "Null",
            Self::String(value)     => &format!("\"{value}\""),
            Self::Integer(value)    => &value.to_string(),
            Self::Decimal(value)    => &value.to_string(),
            Self::Boolean(value)    => &value.to_string(),
            Self::Function(_)       => "Function",
            Self::FunctionNative(_) => "FunctionNative",
            Self::Enumerate(_)      => "Enumerate",
            Self::Structure(_)      => "Structure",
        };

        string.to_string()
    }
}

impl Value {
    pub fn as_string(&self) -> Result<String, Error> {
        if let Self::String(string) = self {
            Ok(string.to_string())
        } else {
            Err(crate::machine::error::Error::IncorrectKind(
                ValueKind::String,
                self.kind(),
            ))
        }
    }

    pub fn as_function(&self) -> Result<Function, Error> {
        if let Self::Function(function) = self {
            Ok(function.clone())
        } else {
            Err(Error::IncorrectKind(ValueKind::Integer, self.kind()))
        }
    }

    pub fn parse_text(kind: &str, text: &str) -> Result<Self, Error> {
        Self::parse_kind(ValueKind::parse_text(kind)?, text)
    }

    pub fn parse_kind(kind: ValueKind, text: &str) -> Result<Self, Error> {
        match kind {
            ValueKind::String => Ok(Self::String(text.to_string())),
            ValueKind::Integer => {
                if let Ok(integer) = text.parse::<i32>() {
                    Ok(Self::Integer(integer))
                } else {
                    Err(Error::IntegerParseFail(text.to_string()))
                }
            }
            ValueKind::Decimal => {
                if let Ok(decimal) = text.parse::<f32>() {
                    Ok(Self::Decimal(decimal))
                } else {
                    Err(Error::DecimalParseFail(text.to_string()))
                }
            }
            ValueKind::Boolean => {
                if text == "true" {
                    Ok(Self::Boolean(true))
                } else if text == "false" {
                    Ok(Self::Boolean(false))
                } else {
                    Err(Error::BooleanParseFail(text.to_string()))
                }
            }
            _ => {
                todo!("Value::parse_text(): TO-DO function/functionNative type.")
            }
        }

        //panic!("Value::parse_text: Unknown value.")
    }

    #[rustfmt::skip]
    pub fn kind(&self) -> ValueKind {
        match self {
            Self::Null                 => ValueKind::Null,
            Self::String(_)            => ValueKind::String,
            Self::Integer(_)           => ValueKind::Integer,
            Self::Decimal(_)           => ValueKind::Decimal,
            Self::Boolean(_)           => ValueKind::Boolean,
            Self::Function(_)          => ValueKind::Function,
            Self::FunctionNative(_)    => ValueKind::FunctionNative,
            Self::Enumerate(_)         => ValueKind::Enumerate,
            Self::Structure(_)         => ValueKind::Structure,
        }
    }
}

//================================================================

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ValueKind {
    Null,
    String,
    Integer,
    Decimal,
    Boolean,
    Function,
    FunctionNative,
    Enumerate,
    Structure,
}

impl ValueKind {
    #[rustfmt::skip]
    fn parse_text(kind: &str) -> Result<Self, Error> {
        match kind {
            "Null"           => Ok(Self::Null),
            "String"         => Ok(Self::String),
            "Integer"        => Ok(Self::Integer),
            "Decimal"        => Ok(Self::Decimal),
            "Boolean"        => Ok(Self::Boolean),
            "Function"       => Ok(Self::Function),
            "FunctionNative" => Ok(Self::FunctionNative),
            "Enumerate"      => Ok(Self::Enumerate),
            "Structure"      => Ok(Self::Structure),
            _                => Err(Error::UnknownKind(kind.to_string())),
        }
    }
}
