use crate::helper::error::Error;
use crate::stage_2::construct::Enumerate as EnumerateD;
use crate::stage_2::construct::Structure as StructureD;
use crate::stage_2::scope::Declaration;
use crate::stage_2::scope::FunctionNative;
use crate::stage_2::scope::Scope;

//================================================================

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

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
        let mut machine = Self {
            function: HashMap::default(),
            structure: HashMap::default(),
            enumerate: HashMap::default(),
        };

        machine.compile(scope)?;

        Ok(machine)
    }

    pub fn compile(&mut self, scope: &Scope) -> Result<(), Error> {
        for value in scope.symbol.clone().values() {
            match value {
                Declaration::Function(f) => {
                    let compile = f.compile(scope)?;

                    if f.name.text == "main" {
                        //for (i, c) in compile.buffer.iter().enumerate() {
                        //    println!("{i}: {c:#?}");
                        //}
                    }

                    self.function
                        .insert(f.name.text.clone(), FunctionKind::Function(compile));
                }
                Declaration::FunctionNative(f) => {
                    self.function
                        .insert(f.name.clone(), FunctionKind::FunctionNative(f.clone()));
                }
                Declaration::Structure(s) => {
                    let mut map = StructureMap::default();

                    for (name, function) in &s.function {
                        let compile = function.compile(scope)?;

                        map.function.insert(name.to_string(), compile);
                    }

                    self.structure.insert(s.name.text.clone(), map);
                }
                Declaration::Enumerate(s) => {
                    let mut map = EnumerateMap::default();

                    for (name, function) in &s.function {
                        let compile = function.compile(scope)?;

                        map.function.insert(name.to_string(), compile);
                    }

                    self.enumerate.insert(s.name.text.clone(), map);
                }
                _ => {}
            }
        }

        Ok(())
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
    table: HashMap<usize, ValuePointer>,
    stack: Vec<ValuePointer>,
    cursor: usize,
}

impl Frame {
    fn save(&mut self, index: usize, value: Value) {
        if self.table.contains_key(&index) {
            let v = self.table.get(&index).unwrap();
            let v = v.borrow_mut().clone();

            match v {
                Value::Reference(ref_cell) => {
                    let mut r = ref_cell.borrow_mut();
                    *r = value;
                }
                _ => {
                    self.table.insert(index, Rc::new(RefCell::new(value)));
                }
            }
        } else {
            self.table.insert(index, Rc::new(RefCell::new(value)));
        }
    }

    fn load(&mut self, index: &usize) -> Value {
        if let Some(value) = self.table.get(index) {
            value.borrow().clone()
        } else {
            panic!("Frame::load(): No value with an index of \"{index}\".")
        }
    }

    fn load_pointer(&mut self, index: &usize) -> ValuePointer {
        if let Some(value) = self.table.get(index) {
            value.clone()
        } else {
            panic!("Frame::load_pointer(): No value with an index of \"{index}\".")
        }
    }

    fn hide(&mut self, index: &usize) {
        self.table.remove(index);
    }

    fn push(&mut self, value: Value) {
        self.stack.push(Rc::new(RefCell::new(value)))
    }

    fn pop(&mut self) -> Value {
        if let Some(pop) = self.stack.pop() {
            pop.borrow().clone()
        } else {
            self.panic("Frame::pop(): No value on the stack.".to_string())
        }
    }

    fn pop_string(&mut self) -> String {
        let pop = self.pop();

        if let Value::String(value) = pop.clone() {
            value
        } else {
            self.panic(format!("Frame::pop_string(): Invalid value \"{pop:?}\"."))
        }
    }

    fn pop_integer(&mut self) -> i64 {
        let pop = self.pop();

        if let Value::Integer(value) = pop {
            value
        } else {
            self.panic(format!("Frame::pop_integer(): Invalid value \"{pop:?}\"."))
        }
    }

    fn pop_decimal(&mut self) -> f64 {
        let pop = self.pop();

        if let Value::Decimal(value) = pop {
            value
        } else {
            self.panic(format!("Frame::pop_decimal(): Invalid value \"{pop:?}\"."))
        }
    }

    fn pop_boolean(&mut self) -> bool {
        let pop = self.pop();

        if let Value::Boolean(value) = pop {
            value
        } else {
            self.panic(format!("Frame::pop_boolean(): Invalid value \"{pop:?}\"."))
        }
    }

    fn pop_structure(&mut self) -> Structure {
        let pop = self.pop();

        if let Value::Structure(value) = pop {
            value
        } else {
            self.panic(format!(
                "Frame::pop_structure(): Invalid value \"{pop:?}\"."
            ))
        }
    }

    fn pop_reference(&mut self) -> ValuePointer {
        let pop = self.pop();

        if let Value::Reference(value) = pop {
            value
        } else {
            self.panic(format!(
                "Frame::pop_structure(): Invalid value \"{pop:?}\"."
            ))
        }
    }

    fn pop_array(&mut self) -> Vec<ValuePointer> {
        let pop = self.pop();

        if let Value::Array(value) = pop {
            value
        } else {
            self.panic(format!("Frame::pop_array(): Invalid value \"{pop:?}\"."))
        }
    }

    fn panic<T>(&self, mut text: String) -> T {
        text.push_str(&format!("\n{:#?}", self));
        panic!("{}", text)
    }
}

//================================================================

type ValuePointer = Rc<RefCell<Value>>;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    Structure(Structure),
    Enumerate(Enumerate),
    Reference(ValuePointer),
    Array(Vec<ValuePointer>),
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub kind: String,
    pub data: HashMap<String, ValuePointer>,
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
                        formatter.write_str(&format!("{k}: {v:?} "))?;
                    } else {
                        formatter.write_str(&format!("{k}: {v:?}, "))?;
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
            Value::Reference(value) => {
                formatter.write_str(&format!("{value:?}"))
            }
            Value::Array(value) => {
                formatter.write_str("[")?;

                let length = value.len();

                for (i, v) in value.iter().enumerate() {
                    if i == length - 1 {
                        formatter.write_str(&format!("{v:?}"))?;
                    } else {
                        formatter.write_str(&format!("{v:?}, "))?;
                    }
                }

                formatter.write_str("]")
            }

        }
    }
}

impl Value {
    pub fn as_string(&self) -> String {
        if let Self::String(value) = self {
            value.to_string()
        } else {
            panic!("Value::as_string(): Invalid value: {}", self)
        }
    }

    pub fn as_integer(&self) -> i64 {
        if let Self::Integer(value) = self {
            *value
        } else {
            panic!("Value::as_integer(): Invalid value: {}", self)
        }
    }

    pub fn as_decimal(&self) -> f64 {
        if let Self::Decimal(value) = self {
            *value
        } else {
            panic!("Value::as_decimal(): Invalid value: {}", self)
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
            Self::Reference(_) => ValueKind::Reference,
            Self::Array(_)     => ValueKind::Array,
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
    Negate,
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
    PushReference(usize),
    PushStructure(StructureD),
    PushEnumerate(EnumerateD, String),
    PushArray(usize),
    //PushTable,
    //PushTuple,
    //PushRange,
    Push(Value),
    Save(usize),
    SaveReference,
    SaveField(String),
    SaveIndex,
    Load(usize),
    LoadField(String),
    LoadIndex,
    Hide(usize),
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
                Instruction::Negate => {
                    let a = frame.pop();

                    match a {
                        Value::Integer(a) => {
                            frame.push(Value::Integer(-a));
                        }
                        Value::Decimal(a) => {
                            frame.push(Value::Decimal(-a));
                        }
                        _ => panic!("Negate: Invalid value {a:?}"),
                    }
                }
                Instruction::PushReference(index) => {
                    let value = frame.load_pointer(index);
                    frame.push(Value::Reference(value));
                }
                Instruction::PushStructure(structure) => {
                    let mut s = Structure::new(structure.name.text.clone());

                    for variable in &structure.variable {
                        s.data
                            .insert(variable.0.to_string(), Rc::new(RefCell::new(frame.pop())));
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
                Instruction::PushArray(arity) => {
                    let mut a = Vec::new();

                    for _ in 0..*arity {
                        a.push(Rc::new(RefCell::new(frame.pop())));
                    }

                    frame.push(Value::Array(a));
                }
                Instruction::Push(value) => {
                    frame.push(value.clone());
                }
                Instruction::Save(index) => {
                    let value = frame.pop();
                    frame.save(*index, value);
                }
                Instruction::SaveReference => {
                    let value_r = frame.pop_reference();
                    let mut value_r = value_r.borrow_mut();
                    let value_v = frame.pop();

                    *value_r = value_v;
                }
                Instruction::SaveField(name) => {
                    let reference = frame.pop_reference().borrow().clone();

                    if let Value::Structure(structure) = reference {
                        let field = structure
                            .data
                            .get(name)
                            .expect(&format!("no field {name} for structure {structure:?}"));

                        frame.push(Value::Reference(field.clone()));
                    }
                }
                Instruction::SaveIndex => {
                    let value = frame.pop_integer();
                    let reference = frame.pop_reference().borrow().clone();

                    if let Value::Array(array) = reference {
                        let field = &array[value as usize];

                        frame.push(Value::Reference(field.clone()));
                    }
                }
                Instruction::Load(index) => {
                    let value = frame.load(index);
                    frame.push(value);
                }
                Instruction::LoadField(name) => {
                    let value = frame.pop_structure();
                    let value = value.data.get(name).unwrap().borrow().clone();
                    frame.push(value);
                }
                Instruction::LoadIndex => {
                    let index = frame.pop_integer();
                    let array = frame.pop_array();
                    frame.push(array[index as usize].borrow().clone());
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
                                (function_native.call)(machine, argument)
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

#[derive(Debug)]
pub struct Argument {
    pub memory: HashMap<usize, ValuePointer>,
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
