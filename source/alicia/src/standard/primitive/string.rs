use crate::machine::Argument;
use crate::machine::Machine;
use crate::machine::Value;
use crate::machine::ValueType;
use crate::scope::FunctionMeta;
use crate::scope::FunctionNative;
use crate::scope::NativeArgument;
use crate::scope::Scope;
use alicia_macro::function;
use alicia_macro::function_string_add;

//================================================================

#[function]
fn to_integer(value: String) -> Integer {
    // TO-DO throw error
    Some(Value::Integer(value.parse().unwrap()))
}

#[function]
fn to_decimal(value: String) -> Decimal {
    // TO-DO throw error
    Some(Value::Decimal(value.parse().unwrap()))
}

#[function]
fn character(value: String, index: i64) -> String {
    // TO-DO throw error
    Some(Value::String(
        value.chars().nth(index as usize).unwrap().to_string(),
    ))
}

// TO-DO add slice/sub-string function

#[function]
fn length(value: String) -> Integer {
    Some(Value::Integer(value.len() as i64))
}

#[function]
fn upper(value: String) -> String {
    Some(Value::String(value.to_uppercase()))
}

#[function]
fn lower(value: String) -> String {
    Some(Value::String(value.to_lowercase()))
}

pub fn module(scope: &mut Scope) {
    function_string_add!(scope, to_integer);
    function_string_add!(scope, to_decimal);
    function_string_add!(scope, character);
    function_string_add!(scope, length);
    function_string_add!(scope, upper);
    function_string_add!(scope, lower);
}
