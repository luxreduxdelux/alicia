use crate::helper::error::Error;
use crate::stage_2::scope::Declaration;
use crate::stage_2::scope::FunctionNative;
use crate::stage_2::scope::Scope;

//================================================================

use std::collections::HashMap;
use std::fmt::Display;

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

                    println!("function: {compile:#?}");

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

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    pub fn pop(&mut self) -> Value {
        if let Some(pop) = self.stack.pop() {
            pop
        } else {
            panic!("Machine::pop(): No element on the stack.")
        }
    }

    pub fn pop_string(&mut self) -> String {
        let pop = self.pop();

        if let Value::String(value) = pop {
            value
        } else {
            panic!("Machine::pop_string(): Invalid value \"{pop:?}\".")
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

    fn pop_boolean(&mut self) -> bool {
        let pop = self.pop();

        if let Value::Boolean(value) = pop {
            value
        } else {
            panic!("Machine::pop_boolean(): Invalid value \"{pop:?}\".")
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

impl Display for Value {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(value)  => formatter.write_str(value),
            Value::Integer(value) => formatter.write_str(&value.to_string()),
            Value::Decimal(value) => formatter.write_str(&value.to_string()),
            Value::Boolean(value) => formatter.write_str(&value.to_string()),
        }
    }
}

impl Value {
    pub fn as_string(&self) -> String {
        if let Self::String(value) = self {
            value.to_string()
        } else {
            panic!("Value::as_string(): Value is not a string.")
        }
    }
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
    Not,
    And,
    Or,
    GT,
    LT,
    Equal,
    GTE,
    LTE,
    EqualNot,
    Push(Value),
    Save(String),
    Load(String),
    Call(String, usize),
}

#[derive(Debug, Clone, Default)]
pub struct Function {
    buffer: Vec<Instruction>,
    enter: Vec<String>,
}

impl Function {
    pub fn push(&mut self, instruction: Instruction) {
        self.buffer.push(instruction);
    }

    pub fn push_parameter(&mut self, parameter: String) {
        self.enter.push(parameter);
    }

    pub fn execute(&self, machine: &mut Machine, argument: Vec<Value>) {
        let mut cursor = 0;
        let mut memory: HashMap<String, Value> = HashMap::default();

        if argument.len() != self.enter.len() {
            panic!("incorrect argument count");
        }

        for (i, a) in argument.iter().enumerate() {
            memory.insert(self.enter[i].clone(), a.clone());
        }

        while let Some(instruction) = self.buffer.get(cursor) {
            match instruction {
                Instruction::Add => {
                    let a = machine.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = machine.pop_integer();
                            machine.push(Value::Integer(b + a));
                        }
                        Value::Decimal(a) => {
                            let b = machine.pop_decimal();
                            machine.push(Value::Decimal(b + a));
                        }
                        _ => panic!("Add: Invalid value {a:?}"),
                    }
                }
                Instruction::Subtract => {
                    let a = machine.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = machine.pop_integer();
                            machine.push(Value::Integer(b - a));
                        }
                        Value::Decimal(a) => {
                            let b = machine.pop_decimal();
                            machine.push(Value::Decimal(b - a));
                        }
                        _ => panic!("Subtract: Invalid value {a:?}"),
                    }
                }
                Instruction::Multiply => {
                    let a = machine.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = machine.pop_integer();
                            machine.push(Value::Integer(b * a));
                        }
                        Value::Decimal(a) => {
                            let b = machine.pop_decimal();
                            machine.push(Value::Decimal(b * a));
                        }
                        _ => panic!("Multiply: Invalid value {a:?}"),
                    }
                }
                Instruction::Divide => {
                    let a = machine.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = machine.pop_integer();
                            machine.push(Value::Integer(b / a));
                        }
                        Value::Decimal(a) => {
                            let b = machine.pop_decimal();
                            machine.push(Value::Decimal(b / a));
                        }
                        _ => panic!("Divide: Invalid value {a:?}"),
                    }
                }
                Instruction::Push(value) => {
                    machine.push(value.clone());
                }
                Instruction::Save(name) => {
                    let value = machine.pop();
                    //machine.save(name.to_string(), value);
                    memory.insert(name.to_string(), value);
                }
                Instruction::Load(name) => {
                    let value = memory.get(name).cloned().unwrap();
                    //let value = machine.load(name);
                    machine.push(value);
                }
                Instruction::Call(name, arity) => {
                    let function = machine.get_function(name);
                    let argument = Argument::new(machine, *arity);

                    match function {
                        FunctionKind::Function(function) => {
                            function.execute(machine, argument.buffer);
                        }
                        FunctionKind::FunctionNative(function_native) => {
                            (function_native.call)(argument);
                        }
                    }
                }
                Instruction::Not => {
                    let a = machine.pop_boolean();
                    machine.push(Value::Boolean(!a));
                }
                Instruction::And => {
                    let a = machine.pop_boolean();
                    let b = machine.pop_boolean();
                    machine.push(Value::Boolean(b && a));
                }
                Instruction::Or => {
                    let a = machine.pop_boolean();
                    let b = machine.pop_boolean();
                    machine.push(Value::Boolean(b || a));
                }
                Instruction::GT => {
                    let a = machine.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = machine.pop_integer();
                            machine.push(Value::Boolean(b > a));
                        }
                        Value::Decimal(a) => {
                            let b = machine.pop_decimal();
                            machine.push(Value::Boolean(b > a));
                        }
                        _ => panic!("GT: Invalid value {a:?}"),
                    }
                }
                Instruction::LT => {
                    let a = machine.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = machine.pop_integer();
                            machine.push(Value::Boolean(b < a));
                        }
                        Value::Decimal(a) => {
                            let b = machine.pop_decimal();
                            machine.push(Value::Boolean(b < a));
                        }
                        _ => panic!("LT: Invalid value {a:?}"),
                    }
                }
                Instruction::Equal => {
                    let a = machine.pop();

                    match a {
                        Value::String(a) => {
                            let b = machine.pop_string();
                            machine.push(Value::Boolean(b == a));
                        }
                        Value::Integer(a) => {
                            let b = machine.pop_integer();
                            machine.push(Value::Boolean(b == a));
                        }
                        Value::Decimal(a) => {
                            let b = machine.pop_decimal();
                            machine.push(Value::Boolean(b == a));
                        }
                        Value::Boolean(a) => {
                            let b = machine.pop_boolean();
                            machine.push(Value::Boolean(b == a));
                        }
                    }
                }
                Instruction::GTE => {
                    let a = machine.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = machine.pop_integer();
                            machine.push(Value::Boolean(b >= a));
                        }
                        Value::Decimal(a) => {
                            let b = machine.pop_decimal();
                            machine.push(Value::Boolean(b >= a));
                        }
                        _ => panic!("GTE: Invalid value {a:?}"),
                    }
                }
                Instruction::LTE => {
                    let a = machine.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = machine.pop_integer();
                            machine.push(Value::Boolean(b <= a));
                        }
                        Value::Decimal(a) => {
                            let b = machine.pop_decimal();
                            machine.push(Value::Boolean(b <= a));
                        }
                        _ => panic!("LTE: Invalid value {a:?}"),
                    }
                }
                Instruction::EqualNot => {
                    let a = machine.pop();

                    match a {
                        Value::String(a) => {
                            let b = machine.pop_string();
                            machine.push(Value::Boolean(b != a));
                        }
                        Value::Integer(a) => {
                            let b = machine.pop_integer();
                            machine.push(Value::Boolean(b != a));
                        }
                        Value::Decimal(a) => {
                            let b = machine.pop_decimal();
                            machine.push(Value::Boolean(b != a));
                        }
                        Value::Boolean(a) => {
                            let b = machine.pop_boolean();
                            machine.push(Value::Boolean(b != a));
                        }
                    }
                }
            }

            cursor += 1;
        }
    }
}

pub struct Argument {
    pub buffer: Vec<Value>,
    cursor: usize,
}

impl Argument {
    fn new(machine: &mut Machine, arity: usize) -> Self {
        let mut buffer = Vec::new();

        for _ in 0..arity {
            buffer.push(machine.pop());
        }

        Self {
            buffer,
            cursor: usize::default(),
        }
    }

    pub fn next(&mut self) -> Option<Value> {
        if let Some(value) = self.buffer.get(self.cursor) {
            self.cursor += 1;
            Some(value.clone())
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cursor == self.buffer.len()
    }
}
