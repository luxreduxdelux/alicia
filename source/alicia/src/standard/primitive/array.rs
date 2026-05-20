use crate::machine::Argument;
use crate::machine::Machine;
use crate::machine::Value;
use crate::machine::ValueType;
use crate::scope::FunctionMeta;
use crate::scope::NativeArgument;
use crate::scope::Scope;
use alicia_macro::function;

//================================================================

fn push(_: &mut Machine, mut argument: Argument) -> Option<Value> {
    let array = argument.next().unwrap();
    let value = argument.next().unwrap();

    println!("enter push: {array:?} : {value:?}");

    if let Value::Reference(array) = array
        && let Value::Array(array) = &mut *array.borrow_mut()
    {
        array.push(value);
    }

    None
}

fn length(_: &mut Machine, mut argument: Argument) -> Option<Value> {
    let array = argument.next().unwrap();

    if let Value::Reference(array) = array
        && let Value::Array(array) = &*array.borrow()
    {
        return Some(array.length().into());
    }

    None
}

pub fn module(scope: &mut Scope) {}
