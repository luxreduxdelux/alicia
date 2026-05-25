use crate::machine::Argument;
use crate::machine::Machine;
use crate::machine::Value;
use crate::machine::ValueType;
use crate::scope::FunctionMeta;
use crate::scope::FunctionNative;
use crate::scope::NativeArgument;
use crate::scope::Scope;
use alicia_macro::function;
use alicia_macro::function_integer_add;
use rand::RngExt;

//================================================================

#[function]
fn to_string(value: i64) -> String {
    Some(Value::String(value.to_string()))
}

#[function]
fn to_decimal(value: i64) -> Decimal {
    Some(Value::Decimal(value as f64))
}

#[function]
fn to_boolean(value: i64) -> Boolean {
    Some(Value::Boolean(value > 0))
}

#[function]
fn sign(value: i64) -> Integer {
    Some(Value::Integer(value.signum()))
}

#[function]
fn absolute(value: i64) -> Integer {
    Some(Value::Integer(value.abs()))
}

#[function]
fn min(value: i64, other: i64) -> Integer {
    Some(Value::Integer(value.min(other)))
}

#[function]
fn max(value: i64, other: i64) -> Integer {
    Some(Value::Integer(value.max(other)))
}

#[function]
fn square_root(value: i64) -> Integer {
    // TO-DO throw error if value is negative
    Some(Value::Integer(value.isqrt()))
}

#[function]
fn random(lower: i64, upper: i64) -> Integer {
    // TO-DO throw error is lower > upper
    Some(Value::Integer(rand::rng().random_range(upper..=lower)))
}

pub fn module(scope: &mut Scope) {
    function_integer_add!(scope, to_string);
    function_integer_add!(scope, to_decimal);
    function_integer_add!(scope, to_boolean);
    function_integer_add!(scope, sign);
    function_integer_add!(scope, absolute);
    function_integer_add!(scope, min);
    function_integer_add!(scope, max);
    function_integer_add!(scope, square_root);
    function_integer_add!(scope, random);
}
