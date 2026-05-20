use crate::machine::Argument;
use crate::machine::Machine;
use crate::machine::Value;
use crate::machine::ValueType;
use crate::scope::FunctionMeta;
use crate::scope::FunctionNative;
use crate::scope::NativeArgument;
use crate::scope::Scope;
use alicia_macro::{function, function_add};

//================================================================

#[function]
fn file_new(path: String, data: String) {
    // TO-DO throw error
    std::fs::write(path, data).unwrap();

    None
}

#[function]
fn file_read(path: String) -> String {
    // TO-DO throw error
    Some(Value::String(std::fs::read_to_string(path).unwrap()))
}

pub fn module(scope: &mut Scope) {
    function_add!(scope, file_new);
    function_add!(scope, file_read);
}
