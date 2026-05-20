use crate::buffer::*;
use crate::construct::definition::Definition;
use crate::construct::enumerate::Enumerate;
use crate::construct::function::Function;
use crate::construct::structure::Structure;
use crate::error::*;
use crate::helper::*;
use crate::machine::Argument;
use crate::machine::Machine;
use crate::machine::Value;
use crate::machine::ValueType;
use crate::standard;
use crate::token::*;

//================================================================

use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

//================================================================

#[derive(Debug, Copy, Clone, Default)]
struct Index {
    variable: usize,
    function: usize,
    structure: usize,
    enumerate: usize,
    function_native: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Method {
    pub string: OrderMap<FunctionNative>,
    pub integer: OrderMap<FunctionNative>,
    pub decimal: OrderMap<FunctionNative>,
    pub array: OrderMap<FunctionNative>,
}

pub type ScopePointer = Rc<RefCell<Scope>>;

#[derive(Debug, Clone)]
pub struct Scope {
    pub source: Vec<Source>,
    pub symbol: OrderMap<Declaration>,
    pub parent: Option<ScopePointer>,
    pub method: Method,
    index: Index,
}

impl Scope {
    pub fn new(parent: Option<ScopePointer>) -> Self {
        let source = if let Some(parent) = &parent {
            parent.borrow().source.clone()
        } else {
            Vec::default()
        };

        let index = if let Some(parent) = &parent {
            parent.borrow().index
        } else {
            Index::default()
        };

        let method = if let Some(parent) = &parent {
            parent.borrow().method.clone()
        } else {
            Method::default()
        };

        Self {
            source,
            symbol: OrderMap::default(),
            parent,
            method,
            index,
        }
    }

    pub fn get_active_source(&self) -> Source {
        self.source.last().unwrap().clone()
    }

    pub fn parse_buffer(&mut self, mut token_buffer: TokenBuffer) -> Result<(), Error> {
        self.source.push(token_buffer.source.clone());

        while let Some(token) = token_buffer.peek() {
            match token.class {
                TokenClass::Function => {
                    let function = Function::parse_token(&mut token_buffer, None)?;
                    self.set_declaration(function.name.clone(), Declaration::Function(function));
                }
                TokenClass::Structure => {
                    let structure = Structure::parse_token(&mut token_buffer)?;
                    self.set_declaration(structure.name.clone(), Declaration::Structure(structure));
                }
                TokenClass::Enumerate => {
                    let enumerate = Enumerate::parse_token(&mut token_buffer)?;
                    self.set_declaration(enumerate.name.clone(), Declaration::Enumerate(enumerate));
                }
                TokenClass::Let => {
                    let definition = Definition::parse_token(&mut token_buffer)?;
                    self.set_declaration(
                        definition.name.clone(),
                        Declaration::Definition(definition),
                    );
                }
                //TokenClass::Use => {
                //    let value = Use::parse_token(&mut token_buffer)?;
                //    println!("use: {value:?}");
                //}
                _ => {
                    return Error::new_info(
                        token_buffer.get_error_info(Some(token.clone())),
                        ErrorKind::UnknownTokenGlobal(token),
                        Some(ErrorHint::Global),
                    );
                }
            };
        }

        Ok(())
    }

    pub fn add_function(&mut self, mut function: FunctionNative) {
        function.index = self.index.function_native;
        self.index.function_native += 1;
        self.symbol
            .insert(function.name.clone(), Declaration::FunctionNative(function));
    }

    pub fn add_function_string(&mut self, mut function: FunctionNative) {
        function.index = self.index.function_native;
        self.index.function_native += 1;
        self.method.string.insert(function.name.clone(), function);
    }

    pub fn add_function_integer(&mut self, mut function: FunctionNative) {
        function.index = self.index.function_native;
        self.index.function_native += 1;
        self.method.integer.insert(function.name.clone(), function);
    }

    pub fn add_function_decimal(&mut self, mut function: FunctionNative) {
        function.index = self.index.function_native;
        self.index.function_native += 1;
        self.method.decimal.insert(function.name.clone(), function);
    }

    pub fn add_function_array(&mut self, mut function: FunctionNative) {
        function.index = self.index.function_native;
        self.index.function_native += 1;
        self.method.array.insert(function.name.clone(), function);
    }

    pub fn analyze(&mut self) -> Result<Self, Error> {
        self.add_function(FunctionNative {
            name: "print".to_string(),
            call: Self::print_native,
            enter: NativeArgument::Variable,
            leave: ValueType::Null,
            index: 0,
        });

        standard::file::module(self);
        standard::system::module(self);
        standard::primitive::module(self);

        //function_add!(scope, test);

        let scope_clone = Rc::new(RefCell::new(self.clone()));

        for value in self.symbol.array.clone() {
            // TO-DO this is quite bad. I think in a future design we should make a difference
            // between pre-analyze declaration and post-analyze declaration rather than modify
            // everything in-place.
            match value {
                Declaration::Function(mut function) => {
                    function.analyze(scope_clone.clone())?;
                    scope_clone
                        .borrow_mut()
                        .set_declaration(function.name.clone(), Declaration::Function(function));
                }
                Declaration::Structure(mut structure) => {
                    structure.analyze(scope_clone.clone())?;
                    scope_clone
                        .borrow_mut()
                        .set_declaration(structure.name.clone(), Declaration::Structure(structure));
                }
                Declaration::Enumerate(mut enumerate) => {
                    enumerate.analyze(scope_clone.clone())?;
                    scope_clone
                        .borrow_mut()
                        .set_declaration(enumerate.name.clone(), Declaration::Enumerate(enumerate));
                }
                Declaration::Definition(mut definition) => {
                    definition.analyze(&mut scope_clone.borrow_mut())?;
                    scope_clone.borrow_mut().set_declaration(
                        definition.name.clone(),
                        Declaration::Definition(definition),
                    );
                }
                _ => {}
            }
        }

        Ok(scope_clone.borrow().clone())
    }

    fn format_internal(mut argument: Argument) -> Result<String, Error> {
        let string = argument.next().unwrap().as_string();
        let mut string = string.chars();
        let mut result = String::default();

        while let Some(character) = string.next() {
            match character {
                '{' => {
                    let mut identifier = String::default();

                    while let Some(character) = string.next() {
                        if character == '}' {
                            break;
                        }

                        identifier.push(character);
                    }

                    //if identifier.is_empty() {
                    if let Some(value) = argument.next() {
                        result.push_str(&value.to_string());
                    }

                    // TO-DO unavailable until a look-up table is made between String -> stack slot..
                    /*
                    } else {
                        let split: Vec<&str> = identifier.split(".").collect();

                        if split.len() == 1 {
                            if let Some(value) = argument.memory.get(&identifier) {
                                result.push_str(&value.to_string());
                            } else {
                                panic!("No variable by the name of {identifier}.")
                            }
                        } else {
                            let mut current = None;

                            for access in split {
                                if let Some(a_current) = &current {
                                    match a_current {
                                        Value::Structure(structure) => {
                                            if let Some(value) = structure.data.get(access) {
                                                current = Some(value.clone());
                                            } else {
                                                panic!(
                                                    "No field in structure {} by the name of {access}.",
                                                    structure.kind
                                                )
                                            }
                                        }
                                        x => current = Some(x.clone()),
                                    }
                                } else {
                                    if let Some(value) = argument.memory.get(access) {
                                        current = Some(value.clone());
                                    } else {
                                        panic!("No variable by the name of {access}.")
                                    }
                                }
                            }

                            if let Some(current) = current {
                                result.push_str(&current.to_string());
                            }
                        }
                    }
                    */
                }
                _ => result.push(character),
            }
        }

        Ok(result)
    }

    fn print_native(machine: &mut Machine, argument: Argument) -> Option<Value> {
        println!("{}", Self::format_internal(argument).unwrap());

        None
    }

    pub fn print(&self) {
        println!("scope: {:#?}", self.symbol);

        if let Some(parent) = &self.parent {
            parent.borrow().print();
        }
    }

    pub fn get_declaration(&self, name: Identifier) -> Option<Declaration> {
        if let Some(declaration) = self.symbol.get(name.text.clone()) {
            Some(declaration.clone())
        } else if let Some(parent) = &self.parent {
            let parent = parent.borrow();
            parent.get_declaration(name)
        } else {
            None
        }
    }

    pub fn get_function_integer(&self, name: Identifier) -> Option<FunctionNative> {
        self.method.integer.get(name.text).cloned()
    }

    pub fn get_function_decimal(&self, name: Identifier) -> Option<FunctionNative> {
        self.method.decimal.get(name.text).cloned()
    }

    pub fn get_function_array(&self, name: Identifier) -> Option<FunctionNative> {
        self.method.array.get(name.text).cloned()
    }

    pub fn get_function(&self, name: Identifier) -> Option<Function> {
        if let Some(d) = self.get_declaration(name)
            && let Declaration::Function(f) = d
        {
            Some(f)
        } else {
            None
        }
    }

    pub fn get_function_native(&self, name: Identifier) -> Option<FunctionNative> {
        if let Some(d) = self.get_declaration(name)
            && let Declaration::FunctionNative(f) = d
        {
            Some(f)
        } else {
            None
        }
    }

    pub fn get_structure(&self, name: Identifier) -> Option<Structure> {
        if let Some(d) = self.get_declaration(name)
            && let Declaration::Structure(s) = d
        {
            Some(s)
        } else {
            None
        }
    }

    pub fn get_enumerate(&self, name: Identifier) -> Option<Enumerate> {
        if let Some(d) = self.get_declaration(name)
            && let Declaration::Enumerate(e) = d
        {
            Some(e)
        } else {
            None
        }
    }

    pub fn get_definition(&self, name: Identifier) -> Option<Definition> {
        if let Some(d) = self.get_declaration(name)
            && let Declaration::Definition(e) = d
        {
            Some(e)
        } else {
            None
        }
    }

    pub fn set_declaration(&mut self, name: Identifier, value: Declaration) {
        self.symbol.insert(name.text, value);
    }

    pub fn get_index_variable(&self) -> usize {
        self.index.variable
    }

    pub fn add_index_variable(&mut self) -> usize {
        self.index.variable += 1;
        self.index.variable - 1
    }

    pub fn add_index_function(&mut self) -> usize {
        self.index.function += 1;
        self.index.function - 1
    }

    pub fn add_index_structure(&mut self) -> usize {
        self.index.structure += 1;
        self.index.structure - 1
    }

    pub fn add_index_enumerate(&mut self) -> usize {
        self.index.enumerate += 1;
        self.index.enumerate - 1
    }
}

#[derive(Debug, Clone)]
pub struct FunctionNative {
    pub name: String,
    pub call: fn(&mut Machine, Argument) -> Option<Value>,
    pub enter: NativeArgument,
    pub leave: ValueType,
    pub index: usize,
}

impl FunctionNative {
    pub fn new(
        name: String,
        call: fn(&mut Machine, Argument) -> Option<Value>,
        enter: NativeArgument,
        leave: ValueType,
        index: usize,
    ) -> Self {
        Self {
            name,
            call,
            enter,
            leave,
            index,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NativeArgument {
    Variable,
    Constant(&'static [ValueType]),
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Function(Function),
    FunctionNative(FunctionNative),
    Structure(Structure),
    Enumerate(Enumerate),
    Definition(Definition),
}

#[derive(Debug)]
pub struct FunctionMeta {
    pub enter: NativeArgument,
    pub leave: ValueType,
}

impl FunctionMeta {
    pub const fn new(enter: NativeArgument, leave: ValueType) -> Self {
        Self { enter, leave }
    }
}
