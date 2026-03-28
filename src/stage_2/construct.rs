use crate::helper::error::*;
use crate::stage_1::buffer::*;
use crate::stage_1::helper::*;
use crate::stage_1::token::*;
use crate::stage_2::scope::*;

//================================================================

#[derive(Debug, Clone)]
pub enum Instruction {
    Function(Function),
    Structure(Structure),
    Enumerate(Enumerate),
    Definition(Definition),
    Assignment(Assignment),
    Invocation(Invocation),
    Condition(Condition),
    Iteration(Iteration),
    Block(Block),
    Skip,
    Exit,
    Return(Return),
}

impl Instruction {
    pub fn parse_token(token: Token, token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        match token.class {
            TokenClass::Function => Ok(Self::Function(Function::parse_token(token_buffer)?)),
            TokenClass::Structure => Ok(Self::Structure(Structure::parse_token(token_buffer)?)),
            TokenClass::Enumerate => Ok(Self::Enumerate(Enumerate::parse_token(token_buffer)?)),
            TokenClass::Let => Ok(Self::Definition(Definition::parse_token(token_buffer)?)),
            TokenClass::If => Ok(Self::Condition(Condition::parse_token(
                token_buffer,
                false,
            )?)),
            TokenClass::Loop => Ok(Self::Iteration(Iteration::parse_token(token_buffer)?)),
            TokenClass::Skip => {
                token_buffer.want(TokenKind::Skip)?;
                Ok(Self::Skip)
            }
            TokenClass::Exit => {
                token_buffer.want(TokenKind::Exit)?;
                Ok(Self::Exit)
            }
            TokenClass::Return => Ok(Self::Return(Return::parse_token(token_buffer)?)),
            TokenClass::String(_) => {
                if let Some(token) = token_buffer.peek_ahead(1)
                    && token.class.kind() == TokenKind::ParenthesisBegin
                {
                    return Ok(Self::Invocation(Invocation::parse_token(token_buffer)?));
                }

                Ok(Self::Assignment(Assignment::parse_token(token_buffer)?))
            }
            TokenClass::CurlyBegin => Ok(Self::Block(Block::parse_token(token_buffer)?)),
            _ => Err(Error::new_info(
                token_buffer.get_error_info(Some(token.clone())),
                ErrorKind::UnknownToken(token),
                Some(ErrorHint::Function),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub value: Option<String>,
    pub block: Block,
    pub child: Option<Box<Condition>>,
}

impl Condition {
    pub fn parse_token(token_buffer: &mut TokenBuffer, recurse: bool) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Condition, |token_buffer| {
            if recurse {
                token_buffer.want(TokenKind::Else)?;

                // else (value) branch.
                if token_buffer.want_peek(TokenKind::String) {
                    let value = Some(token_buffer.want(TokenKind::String)?.class.inner_string());

                    let block = Block::parse_token(token_buffer)?;

                    let child = if token_buffer.want_peek(TokenKind::Else) {
                        Some(Box::new(Self::parse_token(token_buffer, true)?))
                    } else {
                        None
                    };

                    Ok(Self {
                        value,
                        block,
                        child,
                    })
                } else {
                    let block = Block::parse_token(token_buffer)?;

                    Ok(Self {
                        value: None,
                        block,
                        child: None,
                    })
                }
            } else {
                token_buffer.want(TokenKind::If)?;

                let value = Some(token_buffer.want(TokenKind::String)?.class.inner_string());

                let block = Block::parse_token(token_buffer)?;

                let child = if token_buffer.want_peek(TokenKind::Else) {
                    Some(Box::new(Self::parse_token(token_buffer, true)?))
                } else {
                    None
                };

                Ok(Self {
                    value,
                    block,
                    child,
                })
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct Return {
    pub value: Option<Identifier>,
}

impl Return {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Return, |token_buffer| {
            token_buffer.want(TokenKind::Return)?;

            let value = if token_buffer.want_peek(TokenKind::String) {
                Some(token_buffer.want_identifier()?)
            } else {
                None
            };

            Ok(Self { value })
        })
    }
}

#[derive(Debug, Clone)]
pub enum IterationValue {
    Iterational(Assignment),
    Conditional(Identifier),
}

#[derive(Debug, Clone)]
pub struct Iteration {
    pub value: Option<IterationValue>,
    pub block: Block,
}

impl Iteration {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Iteration, |token_buffer| {
            token_buffer.want(TokenKind::Loop)?;

            let value = if token_buffer.want_peek(TokenKind::String) {
                if let Some(token) = token_buffer.peek_ahead(1)
                    && token.class.kind() == TokenKind::Definition
                {
                    Some(IterationValue::Iterational(Assignment::parse_token(
                        token_buffer,
                    )?))
                } else {
                    Some(IterationValue::Conditional(token_buffer.want_identifier()?))
                }
            } else {
                None
            };

            let block = Block::parse_token(token_buffer)?;

            Ok(Self { value, block })
        })
    }
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub span: TokenSpan,
    pub name: Identifier,
    pub kind: Identifier,
    pub value: String,
}

impl Definition {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Definition, |token_buffer| {
            token_buffer.want(TokenKind::Let)?;
            let name = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::Colon)?;
            let kind = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::Definition)?;
            let value = token_buffer.want(TokenKind::String)?.class.inner_string();

            Ok(Self {
                span: token_buffer.get_span(),
                name,
                kind,
                value,
            })
        })
    }

    pub fn analyze(&self, scope: &mut Scope) -> Result<(), Error> {
        scope.set_declaration(self.name.clone(), Declaration::Definition(self.clone()));

        Variable::type_check(&self.span, &self.kind, scope)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: Identifier,
    pub kind: Token,
    pub value: String,
}

impl Assignment {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Assignment, |token_buffer| {
            let name = token_buffer.want_identifier()?;
            let kind = token_buffer.want_definition()?;
            let value = token_buffer.want(TokenKind::String)?.class.inner_string();

            Ok(Self { name, kind, value })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        if let Some(variable) = scope.get_declaration(self.name.clone()) {
            match variable {
                Declaration::Definition(_) => Ok(()),
                _ => {
                    Ok(())
                    // TO-DO throw error on assignment to non-variable
                }
            }
        } else {
            Err(Error::new_kind(
                ErrorKind::UnknownVariable(self.name.clone()),
                Some(ErrorHint::Assignment),
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Invocation {
    pub name: Identifier,
    pub list: Vec<String>,
}

impl Invocation {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Invocation, |token_buffer| {
            let name = token_buffer.want_identifier()?;
            let mut list = Vec::new();

            token_buffer.want(TokenKind::ParenthesisBegin)?;

            while let Some(token) = token_buffer.peek() {
                if token.class.kind() == TokenKind::ParenthesisClose {
                    break;
                }

                if token.class.kind() == TokenKind::Comma {
                    token_buffer.next();
                }

                list.push(token_buffer.want(TokenKind::String)?.class.inner_string());
            }

            token_buffer.want(TokenKind::ParenthesisClose)?;

            Ok(Self { name, list })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        if let Some(declaration) = scope.get_declaration(self.name.clone()) {
            match declaration {
                Declaration::Function(_) => {
                    Ok(())
                    // TO-DO type check and validate the function is receiving every argument
                }
                _ => {
                    todo!()
                    // TO-DO throw error
                }
            }
        } else {
            todo!()
        }
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Block {
    pub code: Vec<Instruction>,
}

impl Block {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Block, |token_buffer| {
            let mut code = Vec::new();

            token_buffer.want(TokenKind::CurlyBegin)?;

            while let Some(token) = token_buffer.peek() {
                if token.class.kind() == TokenKind::CurlyClose {
                    break;
                } else {
                    code.push(Instruction::parse_token(token, token_buffer)?);
                }
            }

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self { code })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        let mut scope_block = Scope::new(Some(Box::new(scope.clone())));

        for instruction in &self.code {
            match instruction {
                Instruction::Function(function) => scope_block.set_declaration(
                    function.name.clone(),
                    Declaration::Function(function.clone()),
                ),
                Instruction::Structure(structure) => scope_block.set_declaration(
                    structure.name.clone(),
                    Declaration::Structure(structure.clone()),
                ),
                Instruction::Enumerate(enumerate) => scope_block.set_declaration(
                    enumerate.name.clone(),
                    Declaration::Enumerate(enumerate.clone()),
                ),
                _ => {}
            }
        }

        for instruction in &self.code {
            match instruction {
                Instruction::Definition(definition) => definition.analyze(&mut scope_block)?,
                Instruction::Assignment(assignment) => assignment.analyze(&scope_block)?,
                Instruction::Invocation(invocation) => invocation.analyze(&scope_block)?,
                /*
                Instruction::Condition(condition) => todo!(),
                Instruction::Iteration(iteration) => todo!(),
                */
                Instruction::Block(block) => block.analyze(&scope_block)?,
                Instruction::Skip => {
                    // TO-DO error: skip outside of interation loop
                }
                Instruction::Exit => {
                    // TO-DO error: exit outside of interation loop
                }
                /*
                Instruction::Return(_) => todo!(),
                */
                _ => {}
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub span: TokenSpan,
    pub name_structure: Option<Identifier>,
    pub name: Identifier,
    pub enter: Vec<Variable>,
    pub leave: Option<Identifier>,
    pub block: Block,
}

impl Function {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Function, |token_buffer| {
            token_buffer.want(TokenKind::Function)?;

            let mut name_structure = None;
            let mut name = token_buffer.want_identifier()?;
            let mut enter = Vec::new();
            let mut leave = None;

            if token_buffer.want_peek(TokenKind::Dot) {
                token_buffer.want(TokenKind::Dot)?;
                name_structure = Some(name);
                name = token_buffer.want_identifier()?;
            }

            token_buffer.want(TokenKind::ParenthesisBegin)?;

            // No argument branch.
            if token_buffer.want_peek(TokenKind::ParenthesisClose) {
                token_buffer.want(TokenKind::ParenthesisClose)?;
            } else {
                while let Some(token) = token_buffer.peek() {
                    if token.class.kind() == TokenKind::ParenthesisClose {
                        break;
                    }

                    if token.class.kind() == TokenKind::Comma {
                        token_buffer.next();
                    }

                    enter.push(Variable::parse_token(token_buffer)?);
                }

                token_buffer.want(TokenKind::ParenthesisClose)?;
            }

            if token_buffer.want_peek(TokenKind::Colon) {
                token_buffer.want(TokenKind::Colon)?;
                leave = Some(token_buffer.want_identifier()?);
            }

            let block = Block::parse_token(token_buffer)?;

            Ok(Self {
                span: token_buffer.get_span(),
                name_structure,
                name,
                enter,
                leave,
                block,
            })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        for variable in &self.enter {
            variable.analyze(scope)?
        }

        if let Some(leave) = &self.leave {
            Variable::type_check(&self.span, leave, scope)?;
        }

        self.block.analyze(scope)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub span: TokenSpan,
    pub name: Identifier,
    pub kind: Identifier,
    pub reference: bool,
}

impl Variable {
    fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Variable, |token_buffer| {
            let name = token_buffer.want_identifier()?;
            token_buffer.want(TokenKind::Colon)?;

            let reference = if token_buffer.want_peek(TokenKind::Ampersand) {
                token_buffer.want(TokenKind::Ampersand)?;
                true
            } else {
                false
            };

            let kind = token_buffer.want_identifier()?;

            Ok(Self {
                span: token_buffer.get_span(),
                name,
                kind,
                reference,
            })
        })
    }

    fn type_check(span: &TokenSpan, name: &Identifier, scope: &Scope) -> Result<(), Error> {
        match name.text.as_str() {
            "String" => {}
            "Integer" => {}
            "Decimal" => {}
            "Boolean" => {}
            "Any" => {}
            _ => {
                if let Some(kind) = scope.get_declaration(name.clone()) {
                    match kind {
                        Declaration::Structure(_) => return Ok(()),
                        Declaration::Enumerate(_) => return Ok(()),
                        _ => {
                            // TO-DO throw error on unsupported construct for variable type.
                        }
                    }
                }

                return Err(Error::new_info(
                    ErrorInfo::new(span.clone(), None),
                    ErrorKind::UnknownKind(name.text.clone()),
                    Some(ErrorHint::Variable),
                ));
            }
        }

        Ok(())
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        Self::type_check(&self.span, &self.kind, scope)
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Structure {
    pub name: Identifier,
    pub list: Vec<Variable>,
}

impl Structure {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Structure, |token_buffer| {
            let mut list = Vec::new();

            token_buffer.want(TokenKind::Structure)?;

            let name = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::CurlyBegin)?;

            while let Some(token) = token_buffer.peek() {
                if token.class.kind() == TokenKind::CurlyClose {
                    break;
                }

                if token.class.kind() == TokenKind::Comma {
                    token_buffer.next();
                }

                list.push(Variable::parse_token(token_buffer)?);
            }

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self { name, list })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        for variable in &self.list {
            variable.analyze(scope)?
        }

        Ok(())
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Enumerate {
    pub name: Identifier,
    pub list: Vec<Identifier>,
}

impl Enumerate {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Enumerate, |token_buffer| {
            let mut list = Vec::new();

            token_buffer.want(TokenKind::Enumerate)?;

            let name = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::CurlyBegin)?;

            while let Some(token) = token_buffer.peek() {
                if token.class.kind() == TokenKind::CurlyClose {
                    break;
                }

                if token.class.kind() == TokenKind::Comma {
                    token_buffer.next();
                }

                list.push(token_buffer.want_identifier()?);
            }

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self { name, list })
        })
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Use {
    pub path: Path,
}

impl Use {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Use, |token_buffer| {
            let mut path = Path::default();

            while let Some(token) = token_buffer.peek() {
                if token.class.kind() == TokenKind::String {
                    path.push(token_buffer.want_identifier()?);
                } else if token.class.kind() == TokenKind::Dot {
                    token_buffer.want(TokenKind::Dot)?;
                } else {
                    break;
                }
            }

            Ok(Self { path })
        })
    }
}
