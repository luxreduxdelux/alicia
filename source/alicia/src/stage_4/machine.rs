use crate::helper::error::Error;
use crate::stage_2::scope::Declaration;
use crate::stage_2::scope::FunctionNative;
use crate::stage_2::scope::Scope;

//================================================================

use std::collections::HashMap;

//================================================================

#[derive(Debug, Clone)]
pub enum FunctionKind {
    Function(Function),
    FunctionNative(FunctionNative),
}

#[derive(Debug, Clone)]
pub struct Machine {
    pub function: HashMap<String, FunctionKind>,
    table: HashMap<String, Value>,
    stack: Vec<Value>,
}

impl Machine {
    pub fn new(scope: &Scope) -> Result<Self, Error> {
        let mut function = HashMap::default();
        let table = HashMap::default();
        let stack = Vec::default();

        for value in scope.symbol.clone().values() {
            match value {
                Declaration::Function(f) => {
                    let compile = f.compile(scope)?;

                    //println!("function: {compile:#?}");

                    function.insert(f.name.text.clone(), FunctionKind::Function(compile));
                }
                Declaration::FunctionNative(f) => {
                    //println!("function native: {f:#?}");

                    function.insert(f.name.clone(), FunctionKind::FunctionNative(f.clone()));
                }
                _ => {}
            }
        }

        Ok(Self {
            function,
            table,
            stack,
        })
    }

    fn set_function(&mut self, name: String, function: FunctionKind) {
        self.function.insert(name, function);
    }

    fn get_function(&self, name: &str) -> FunctionKind {
        if let Some(value) = self.function.get(name) {
            value.clone()
        } else {
            panic!("Machine::get_function(): No function by the name of \"{name}\".")
        }
    }

    fn save(&mut self, name: String, value: Value) {
        self.table.insert(name, value);
    }

    fn load(&mut self, name: &str) -> Value {
        if let Some(value) = self.table.get(name) {
            value.clone()
        } else {
            panic!("Machine::load(): No value by the name of \"{name}\".")
        }
    }

    fn hide(&mut self, name: &str) {
        if self.table.contains_key(name) {
            self.table.remove(name);
        } else {
            panic!("Machine::hide(): No value by the name of \"{name}\".")
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn pop(&mut self) -> Value {
        if let Some(pop) = self.stack.pop() {
            pop
        } else {
            panic!("Machine::pop(): No element on the stack.")
        }
    }

    fn pop_integer(&mut self) -> i64 {
        let pop = self.pop();

        if let Value::Integer(value) = pop {
            value
        } else {
            panic!("Machine::pop_integer(): Invalid value \"{pop:?}\".")
        }
    }

    fn pop_decimal(&mut self) -> f64 {
        let pop = self.pop();

        if let Value::Decimal(value) = pop {
            value
        } else {
            panic!("Machine::pop_decimal(): Invalid value \"{pop:?}\".")
        }
    }
}

//================================================================

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum ValueKind {
    String,
    Integer,
    Decimal,
    Boolean,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Add,
    Subtract,
    Multiply,
    Divide,
    Push(Value),
    Save(String),
    Load(String),
    Hide(String),
    Call(String),
}

#[derive(Debug, Clone, Default)]
pub struct Function {
    buffer: Vec<Instruction>,
}

impl Function {
    pub fn push(&mut self, instruction: Instruction) {
        self.buffer.push(instruction);
    }

    pub fn execute(&self, machine: &mut Machine) {
        let mut cursor = 0;

        while let Some(instruction) = self.buffer.get(cursor) {
            match instruction {
                Instruction::Add => {
                    let a = machine.pop_integer();
                    let b = machine.pop_integer();
                    machine.push(Value::Integer(a + b));
                }
                Instruction::Subtract => {
                    let a = machine.pop_integer();
                    let b = machine.pop_integer();
                    machine.push(Value::Integer(a - b));
                }
                Instruction::Multiply => {
                    let a = machine.pop_integer();
                    let b = machine.pop_integer();
                    machine.push(Value::Integer(a * b));
                }
                Instruction::Divide => {
                    let a = machine.pop_integer();
                    let b = machine.pop_integer();
                    machine.push(Value::Integer(a / b));
                }
                Instruction::Push(value) => {
                    machine.push(value.clone());
                }
                Instruction::Save(name) => {
                    let value = machine.pop();
                    machine.save(name.to_string(), value);
                }
                Instruction::Load(name) => {
                    let value = machine.load(name);
                    machine.push(value);
                }
                Instruction::Hide(name) => {
                    machine.hide(name);
                }
                Instruction::Call(name) => {
                    let function = machine.get_function(name);

                    match function {
                        FunctionKind::Function(function) => {
                            function.execute(machine);
                        }
                        FunctionKind::FunctionNative(function_native) => {
                            (function_native.call)();
                        }
                    }
                }
            }

            cursor += 1;
        }
    }
}
