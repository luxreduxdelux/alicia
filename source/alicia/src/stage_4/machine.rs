use crate::helper::error::Error;
use crate::stage_2::construct::Enumerate as EnumerateD;
use crate::stage_2::construct::Structure as StructureD;
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

#[derive(Debug, Clone, Default)]
struct StructureMap {
    function: HashMap<String, Function>,
}

#[derive(Debug, Clone, Default)]
struct EnumerateMap {
    function: HashMap<String, Function>,
}

#[derive(Debug, Clone)]
pub struct Machine {
    pub function: HashMap<String, FunctionKind>,
    structure: HashMap<String, StructureMap>,
    enumerate: HashMap<String, EnumerateMap>,
}

impl Machine {
    pub fn new(scope: &Scope) -> Result<Self, Error> {
        let mut function = HashMap::default();
        let mut structure = HashMap::default();
        let mut enumerate = HashMap::default();

        for value in scope.symbol.clone().values() {
            match value {
                Declaration::Function(f) => {
                    let compile = f.compile(scope)?;

                    function.insert(f.name.text.clone(), FunctionKind::Function(compile));
                }
                Declaration::FunctionNative(f) => {
                    function.insert(f.name.clone(), FunctionKind::FunctionNative(f.clone()));
                }
                Declaration::Structure(s) => {
                    let mut map = StructureMap::default();

                    for (name, function) in &s.function {
                        let compile = function.compile(scope)?;

                        map.function.insert(name.to_string(), compile);
                    }

                    structure.insert(s.name.text.clone(), map);
                }
                Declaration::Enumerate(s) => {
                    let mut map = EnumerateMap::default();

                    for (name, function) in &s.function {
                        let compile = function.compile(scope)?;

                        map.function.insert(name.to_string(), compile);
                    }

                    enumerate.insert(s.name.text.clone(), map);
                }
                _ => {}
            }
        }

        Ok(Self {
            function,
            structure,
            enumerate,
        })
    }

    fn get_function(&self, name: &str) -> FunctionKind {
        if let Some(value) = self.function.get(name) {
            value.clone()
        } else {
            panic!("Machine::get_function(): No function by the name of \"{name}\".")
        }
    }

    fn get_structure(&self, name: &str) -> StructureMap {
        if let Some(value) = self.structure.get(name) {
            value.clone()
        } else {
            panic!("Machine::get_structure(): No structure by the name of \"{name}\".")
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Frame {
    table: HashMap<usize, Value>,
    stack: Vec<Value>,
    cursor: usize,
}

impl Frame {
    fn save(&mut self, index: usize, value: Value) {
        self.table.insert(index, value);
    }

    fn load(&mut self, index: &usize) -> Value {
        if let Some(value) = self.table.get(index) {
            value.clone()
        } else {
            panic!("Machine::load(): No value with an index of \"{index}\".")
        }
    }

    fn hide(&mut self, index: &usize) {
        self.table.remove(index);
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn pop(&mut self) -> Value {
        if let Some(pop) = self.stack.pop() {
            pop
        } else {
            panic!("Machine::pop(): No element on the stack. \n{:#?}", self)
        }
    }

    fn pop_string(&mut self) -> String {
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

    fn pop_structure(&mut self) -> Structure {
        let pop = self.pop();

        if let Value::Structure(value) = pop {
            value
        } else {
            panic!("Machine::pop_structure(): Invalid value \"{pop:?}\".")
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
    Structure(Structure),
    Enumerate(Enumerate),
    //Array(Vec<Value>),
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub kind: String,
    pub data: HashMap<String, Value>,
}

impl Structure {
    fn new(kind: String) -> Self {
        Self {
            kind,
            data: HashMap::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Enumerate {
    pub name: String,
    pub kind: String,
    pub data: Vec<Value>,
}

impl Enumerate {
    fn new(name: String, kind: String) -> Self {
        Self {
            name,
            kind,
            data: Vec::default(),
        }
    }
}

impl Display for Value {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(value)    => formatter.write_str(value),
            Value::Integer(value)   => formatter.write_str(&value.to_string()),
            Value::Decimal(value)   => formatter.write_str(&value.to_string()),
            Value::Boolean(value)   => formatter.write_str(&value.to_string()),
            Value::Structure(value) => {
                formatter.write_str(&value.kind)?;

                formatter.write_str(" { ")?;

                let length = value.data.len();

                for (i, (k, v)) in value.data.iter().enumerate() {
                    if i == length - 1 {
                        formatter.write_str(&format!("{k}: {v} "))?;
                    } else {
                        formatter.write_str(&format!("{k}: {v}, "))?;
                    }
                }

                formatter.write_str("}")
            },
            Value::Enumerate(value) => {
                formatter.write_str(&value.name)?;
                formatter.write_str(" : ")?;
                formatter.write_str(&value.kind)?;

                formatter.write_str(" { ")?;

                let length = value.data.len();

                for (i, v) in value.data.iter().enumerate() {
                    if i == length - 1 {
                        formatter.write_str(&format!("{v} "))?;
                    } else {
                        formatter.write_str(&format!("{v}, "))?;
                    }
                }

                formatter.write_str("}")
            },
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

    #[rustfmt::skip]
    pub fn kind(&self) -> ValueKind {
        match self {
            Self::String(_)    => ValueKind::String,
            Self::Integer(_)   => ValueKind::Integer,
            Self::Decimal(_)   => ValueKind::Decimal,
            Self::Boolean(_)   => ValueKind::Boolean,
            Self::Structure(_) => ValueKind::Structure,
            Self::Enumerate(_) => ValueKind::Enumerate,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValueKind {
    String,
    Integer,
    Decimal,
    Boolean,
    Structure,
    Enumerate,
    Reference,
    Array,
    Table,
}

impl Display for ValueKind {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String    => formatter.write_str("String"),
            Self::Integer   => formatter.write_str("Integer"),
            Self::Decimal   => formatter.write_str("Decimal"),
            Self::Boolean   => formatter.write_str("Boolean"),
            Self::Structure => formatter.write_str("Structure"),
            Self::Enumerate => formatter.write_str("Enumerate"),
            Self::Reference => formatter.write_str("Reference"),
            Self::Array     => formatter.write_str("Array"),
            Self::Table     => formatter.write_str("Table"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Null,
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
    Jump(usize),
    Branch(usize),
    Return(bool),
    PushStructure(StructureD),
    PushEnumerate(EnumerateD, String),
    Push(Value),
    Save(usize),
    SaveField(String),
    Load(usize),
    LoadField(String),
    Hide(usize),
    //LoadIndex(usize),
    Call(FunctionCall, usize),
}

#[derive(Debug, Clone)]
pub enum FunctionCall {
    Function(String),
    FunctionStructure(String, String),
    FunctionEnumerate(String, String),
}

#[derive(Debug, Clone, Default)]
pub struct Function {
    pub buffer: Vec<Instruction>,
    enter: Vec<String>,
}

impl Function {
    pub fn push(&mut self, instruction: Instruction) {
        self.buffer.push(instruction);
    }

    pub fn insert(&mut self, instruction: Instruction, index: usize) {
        self.buffer.insert(index, instruction);
    }

    pub fn change(&mut self, instruction: Instruction, index: usize) {
        self.buffer[index] = instruction;
    }

    pub fn cursor(&self) -> usize {
        self.buffer.len()
    }

    pub fn push_parameter(&mut self, parameter: String) {
        self.enter.push(parameter);
    }

    pub fn execute(&self, machine: &mut Machine, argument: Vec<Value>) -> Option<Value> {
        let mut frame = Frame::default();

        if argument.len() != self.enter.len() {
            panic!("incorrect argument count");
        }

        for (i, a) in argument.iter().enumerate() {
            frame.save(i, a.clone());
        }

        while let Some(instruction) = self.buffer.get(frame.cursor) {
            match instruction {
                Instruction::Null => {}
                Instruction::Add => {
                    let a = frame.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = frame.pop_integer();
                            frame.push(Value::Integer(b + a));
                        }
                        Value::Decimal(a) => {
                            let b = frame.pop_decimal();
                            frame.push(Value::Decimal(b + a));
                        }
                        _ => panic!("Add: Invalid value {a:?}"),
                    }
                }
                Instruction::Subtract => {
                    let a = frame.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = frame.pop_integer();
                            frame.push(Value::Integer(b - a));
                        }
                        Value::Decimal(a) => {
                            let b = frame.pop_decimal();
                            frame.push(Value::Decimal(b - a));
                        }
                        _ => panic!("Subtract: Invalid value {a:?}"),
                    }
                }
                Instruction::Multiply => {
                    let a = frame.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = frame.pop_integer();
                            frame.push(Value::Integer(b * a));
                        }
                        Value::Decimal(a) => {
                            let b = frame.pop_decimal();
                            frame.push(Value::Decimal(b * a));
                        }
                        _ => panic!("Multiply: Invalid value {a:?}"),
                    }
                }
                Instruction::Divide => {
                    let a = frame.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = frame.pop_integer();
                            frame.push(Value::Integer(b / a));
                        }
                        Value::Decimal(a) => {
                            let b = frame.pop_decimal();
                            frame.push(Value::Decimal(b / a));
                        }
                        _ => panic!("Divide: Invalid value {a:?}"),
                    }
                }
                Instruction::PushStructure(structure) => {
                    let mut s = Structure::new(structure.name.text.clone());

                    for variable in &structure.variable {
                        s.data.insert(variable.0.to_string(), frame.pop());
                    }

                    frame.push(Value::Structure(s));
                }
                Instruction::PushEnumerate(enumerate, kind) => {
                    let k = enumerate.variable.get(kind).unwrap();
                    let mut e = Enumerate::new(enumerate.name.text.clone(), kind.to_string());

                    for _ in k {
                        e.data.push(frame.pop());
                    }

                    frame.push(Value::Enumerate(e));
                }
                Instruction::Push(value) => {
                    frame.push(value.clone());
                }
                Instruction::Save(index) => {
                    let value = frame.pop();
                    frame.save(*index, value);
                }
                Instruction::SaveField(name) => {
                    let mut structure = frame.pop_structure();
                    let value = frame.pop();
                    structure.data.insert(name.to_string(), value);
                    frame.push(Value::Structure(structure));
                }
                Instruction::Load(index) => {
                    let value = frame.load(index);
                    frame.push(value);
                }
                Instruction::LoadField(name) => {
                    let value = frame.pop_structure();
                    let value = value.data.get(name).unwrap();
                    frame.push(value.clone());
                }
                Instruction::Hide(index) => {
                    frame.hide(index);
                }
                Instruction::Call(call, arity) => match call {
                    FunctionCall::Function(name) => {
                        let function = machine.get_function(name);
                        let argument = Argument::new(&mut frame, *arity);

                        let value = match function {
                            FunctionKind::Function(function) => {
                                function.execute(machine, argument.buffer)
                            }
                            FunctionKind::FunctionNative(function_native) => {
                                (function_native.call)(argument)
                            }
                        };

                        if let Some(value) = value {
                            frame.push(value);
                        }
                    }
                    FunctionCall::FunctionStructure(structure, name) => {
                        let structure = machine.get_structure(structure);
                        let function = structure.function.get(name).unwrap();
                        let argument = Argument::new(&mut frame, *arity);

                        let value = function.execute(machine, argument.buffer);

                        if let Some(value) = value {
                            frame.push(value);
                        }
                    }
                    FunctionCall::FunctionEnumerate(enumerate, name) => todo!(),
                },
                Instruction::Not => {
                    let a = frame.pop_boolean();
                    frame.push(Value::Boolean(!a));
                }
                Instruction::And => {
                    let a = frame.pop_boolean();
                    let b = frame.pop_boolean();
                    frame.push(Value::Boolean(b && a));
                }
                Instruction::Or => {
                    let a = frame.pop_boolean();
                    let b = frame.pop_boolean();
                    frame.push(Value::Boolean(b || a));
                }
                Instruction::GT => {
                    let a = frame.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = frame.pop_integer();
                            frame.push(Value::Boolean(b > a));
                        }
                        Value::Decimal(a) => {
                            let b = frame.pop_decimal();
                            frame.push(Value::Boolean(b > a));
                        }
                        _ => panic!("GT: Invalid value {a:?}"),
                    }
                }
                Instruction::LT => {
                    let a = frame.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = frame.pop_integer();
                            frame.push(Value::Boolean(b < a));
                        }
                        Value::Decimal(a) => {
                            let b = frame.pop_decimal();
                            frame.push(Value::Boolean(b < a));
                        }
                        _ => panic!("LT: Invalid value {a:?}"),
                    }
                }
                Instruction::Equal => {
                    let a = frame.pop();

                    match a {
                        Value::String(a) => {
                            let b = frame.pop_string();
                            frame.push(Value::Boolean(b == a));
                        }
                        Value::Integer(a) => {
                            let b = frame.pop_integer();
                            frame.push(Value::Boolean(b == a));
                        }
                        Value::Decimal(a) => {
                            let b = frame.pop_decimal();
                            frame.push(Value::Boolean(b == a));
                        }
                        Value::Boolean(a) => {
                            let b = frame.pop_boolean();
                            frame.push(Value::Boolean(b == a));
                        }
                        _ => todo!(),
                    }
                }
                Instruction::GTE => {
                    let a = frame.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = frame.pop_integer();
                            frame.push(Value::Boolean(b >= a));
                        }
                        Value::Decimal(a) => {
                            let b = frame.pop_decimal();
                            frame.push(Value::Boolean(b >= a));
                        }
                        _ => panic!("GTE: Invalid value {a:?}"),
                    }
                }
                Instruction::LTE => {
                    let a = frame.pop();

                    match a {
                        Value::Integer(a) => {
                            let b = frame.pop_integer();
                            frame.push(Value::Boolean(b <= a));
                        }
                        Value::Decimal(a) => {
                            let b = frame.pop_decimal();
                            frame.push(Value::Boolean(b <= a));
                        }
                        _ => panic!("LTE: Invalid value {a:?}"),
                    }
                }
                Instruction::EqualNot => {
                    let a = frame.pop();

                    match a {
                        Value::String(a) => {
                            let b = frame.pop_string();
                            frame.push(Value::Boolean(b != a));
                        }
                        Value::Integer(a) => {
                            let b = frame.pop_integer();
                            frame.push(Value::Boolean(b != a));
                        }
                        Value::Decimal(a) => {
                            let b = frame.pop_decimal();
                            frame.push(Value::Boolean(b != a));
                        }
                        Value::Boolean(a) => {
                            let b = frame.pop_boolean();
                            frame.push(Value::Boolean(b != a));
                        }
                        _ => todo!(),
                    }
                }
                Instruction::Jump(j_cursor) => {
                    frame.cursor = *j_cursor;
                    continue;
                }
                Instruction::Branch(b_cursor) => {
                    let value = frame.pop_boolean();

                    if !value {
                        frame.cursor = *b_cursor;
                    }
                }
                Instruction::Return(value) => {
                    if *value {
                        let value = frame.pop();
                        return Some(value);
                    } else {
                        return None;
                    }
                }
            }

            frame.cursor += 1;
        }

        None
    }
}

pub struct Argument {
    pub memory: HashMap<usize, Value>,
    pub buffer: Vec<Value>,
    cursor: usize,
}

impl Argument {
    fn new(frame: &mut Frame, arity: usize) -> Self {
        let mut buffer = Vec::new();

        for _ in 0..arity {
            buffer.push(frame.pop());
        }

        Self {
            memory: frame.table.clone(),
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
