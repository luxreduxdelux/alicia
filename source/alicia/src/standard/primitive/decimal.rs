use crate::machine::Argument;
use crate::machine::Machine;
use crate::machine::Value;
use crate::machine::ValueType;
use crate::scope::FunctionMeta;
use crate::scope::FunctionNative;
use crate::scope::NativeArgument;
use crate::scope::Scope;
use alicia_macro::function;
use alicia_macro::function_decimal_add;

//================================================================

#[function]
fn to_string(value: f64) -> String {
    Some(Value::String(value.to_string()))
}

#[function]
fn to_integer(value: f64) -> Integer {
    Some(Value::Integer(value as i64))
}

#[function]
fn to_boolean(value: f64) -> Decimal {
    Some(Value::Boolean(value > 0.0))
}

#[function]
fn sign(value: f64) -> Decimal {
    Some(Value::Decimal(value.signum()))
}

#[function]
fn absolute(value: f64) -> Decimal {
    Some(Value::Decimal(value.abs()))
}

#[function]
fn min(value: f64, other: f64) -> Decimal {
    Some(Value::Decimal(value.min(other)))
}

#[function]
fn max(value: f64, other: f64) -> Decimal {
    Some(Value::Decimal(value.max(other)))
}

#[function]
fn square_root(value: f64) -> Decimal {
    // TO-DO throw error if value is negative
    Some(Value::Decimal(value.sqrt()))
}

#[function]
fn sin(value: f64) -> Decimal {
    Some(Value::Decimal(value.sin()))
}

#[function]
fn cos(value: f64) -> Decimal {
    Some(Value::Decimal(value.cos()))
}

#[function]
fn tan(value: f64) -> Decimal {
    Some(Value::Decimal(value.tan()))
}

#[function]
fn asin(value: f64) -> Decimal {
    Some(Value::Decimal(value.asin()))
}

#[function]
fn acos(value: f64) -> Decimal {
    Some(Value::Decimal(value.acos()))
}

#[function]
fn atan(value: f64) -> Decimal {
    Some(Value::Decimal(value.atan()))
}

#[function]
fn round(value: f64) -> Decimal {
    Some(Value::Decimal(value.round()))
}

#[function]
fn above(value: f64) -> Decimal {
    Some(Value::Decimal(value.ceil()))
}

#[function]
fn below(value: f64) -> Decimal {
    Some(Value::Decimal(value.floor()))
}

pub fn module(scope: &mut Scope) {
    function_decimal_add!(scope, to_string);
    function_decimal_add!(scope, to_integer);
    function_decimal_add!(scope, to_boolean);
    function_decimal_add!(scope, sign);
    function_decimal_add!(scope, absolute);
    function_decimal_add!(scope, min);
    function_decimal_add!(scope, max);
    function_decimal_add!(scope, square_root);
    function_decimal_add!(scope, sin);
    function_decimal_add!(scope, cos);
    function_decimal_add!(scope, tan);
    function_decimal_add!(scope, asin);
    function_decimal_add!(scope, acos);
    function_decimal_add!(scope, atan);
    function_decimal_add!(scope, round);
    function_decimal_add!(scope, above);
    function_decimal_add!(scope, below);
}
