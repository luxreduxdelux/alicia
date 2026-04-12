use crate::helper::error::*;
use crate::stage_1::buffer::*;
use crate::stage_1::helper::*;
use crate::stage_1::token::*;
use crate::stage_2::scope::*;
use crate::stage_4::buffer::ArgumentBuffer;

//================================================================

use std::collections::HashMap;

use std::fmt::Display;
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
    fn parse_identifier(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        if let Some(token) = token_buffer.peek_ahead(1)
            && token.class.kind() == TokenKind::ParenthesisBegin
        {
            return Ok(Self::Invocation(Invocation::parse_token(token_buffer)?));
        }

        Ok(Self::Assignment(Assignment::parse_token(token_buffer)?))
    }

    fn parse_comma<F: FnMut(&mut TokenBuffer) -> Result<(), Error>>(
        token_buffer: &mut TokenBuffer,
        delimiter: TokenKind,
        mut call: F,
    ) -> Result<(), Error> {
        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == delimiter {
                break;
            }

            //list.push(token_buffer.want_identifier()?);
            call(token_buffer)?;

            if let Some(token) = token_buffer.peek()
                && token.class.kind() == TokenKind::Comma
            {
                token_buffer.next();
            } else {
                break;
            }
        }

        Ok(())
    }

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
            TokenClass::Identifier(_) => Self::parse_identifier(token_buffer),
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
    pub value: Option<Expression>,
    pub block: Block,
    pub child: Option<Box<Condition>>,
}

impl Condition {
    pub fn parse_token(token_buffer: &mut TokenBuffer, recurse: bool) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Condition, |token_buffer| {
            if recurse {
                token_buffer.want(TokenKind::Else)?;

                // else (value) branch.
                if token_buffer.want_peek(TokenKind::CurlyBegin) {
                    let block = Block::parse_token(token_buffer)?;

                    Ok(Self {
                        value: None,
                        block,
                        child: None,
                    })
                } else {
                    let value = Expression::parse_token(token_buffer, 0.0)?; //Some(token_buffer.want(TokenKind::String)?.class.inner_string());
                    let block = Block::parse_token(token_buffer)?;

                    let child = if token_buffer.want_peek(TokenKind::Else) {
                        Some(Box::new(Self::parse_token(token_buffer, true)?))
                    } else {
                        None
                    };

                    Ok(Self {
                        value: Some(value),
                        block,
                        child,
                    })
                }
            } else {
                token_buffer.want(TokenKind::If)?;

                let value = Expression::parse_token(token_buffer, 0.0)?;
                let block = Block::parse_token(token_buffer)?;

                let child = if token_buffer.want_peek(TokenKind::Else) {
                    Some(Box::new(Self::parse_token(token_buffer, true)?))
                } else {
                    None
                };

                Ok(Self {
                    value: Some(value),
                    block,
                    child,
                })
            }
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        Ok(())
    }

    pub fn execute(&self, scope: &Scope) -> Result<(), Error> {
        if let Some(value) = &self.value {
            let result = value.evaluate(scope)?;

            match result {
                Some(result) => match result {
                    Value::Boolean(result) => {
                        if result {
                            self.block.execute(scope)?;
                        } else if let Some(child) = &self.child {
                            child.execute(scope)?;
                        }
                    }
                    _ => todo!(),
                },
                None => todo!(),
            }
        } else {
            self.block.execute(scope)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Return {
    pub value: Option<Expression>,
}

impl Return {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Return, |token_buffer| {
            token_buffer.want(TokenKind::Return)?;

            let value = if token_buffer.want_peek(TokenKind::ColonSemi) {
                None
            } else {
                Some(Expression::parse_token(token_buffer, 0.0)?)
            };

            token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self { value })
        })
    }
}

#[derive(Debug, Clone)]
pub enum IterationValue {
    Iterational(Assignment),
    Conditional(Expression),
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

            let value = if token_buffer.want_peek(TokenKind::Identifier) {
                if let Some(token) = token_buffer.peek_ahead(1)
                    && token.class.kind() == TokenKind::Definition
                {
                    Some(IterationValue::Iterational(Assignment::parse_token(
                        token_buffer,
                    )?))
                } else {
                    Some(IterationValue::Conditional(Expression::parse_token(
                        token_buffer,
                        0.0,
                    )?))
                }
            } else {
                None
            };

            let block = Block::parse_token(token_buffer)?;

            Ok(Self { value, block })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        if let Some(value) = &self.value {
            match value {
                IterationValue::Iterational(assignment) => assignment.analyze(scope)?,
                IterationValue::Conditional(expression) => expression.analyze(scope)?,
            };
        }

        self.block.analyze(scope, true)?;

        Ok(())
    }

    pub fn execute(&self, scope: &Scope) -> Result<(), Error> {
        if let Some(value) = &self.value {
            match value {
                IterationValue::Iterational(assignment) => todo!(),
                IterationValue::Conditional(expression) => loop {
                    let result = expression.evaluate(scope)?;

                    match result {
                        Some(result) => match result {
                            Value::Boolean(result) => {
                                if result {
                                    self.block.execute(scope)?;
                                } else {
                                    break;
                                }
                            }
                            _ => todo!(),
                        },
                        None => todo!(),
                    }
                },
            }
        } else {
            loop {
                let (result, exit) = self.block.execute_loop(scope)?;

                if let Some(exit) = exit
                    && exit
                {
                    break;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionKind {
    Path,
    String,
    Integer,
    Decimal,
    Boolean,
    Structure,
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    Structure(StructureV),
}

impl Display for Value {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value)    => formatter.write_str(&value.to_string()),
            Self::Integer(value)   => formatter.write_str(&value.to_string()),
            Self::Decimal(value)   => formatter.write_str(&value.to_string()),
            Self::Boolean(value)   => formatter.write_str(&value.to_string()),
            Self::Structure(value) => formatter.write_str(&value.to_string()),
        }
    }
}

impl Value {
    pub fn as_string(&self) -> Result<String, Error> {
        match self {
            Self::String(value) => Ok(value.to_string()),
            _ => panic!("value is not a decimal"),
        }
    }

    pub fn as_integer(&self) -> Result<i64, Error> {
        match self {
            Self::Integer(value) => Ok(*value),
            _ => panic!("value is not an integer"),
        }
    }

    pub fn as_decimal(&self) -> Result<f64, Error> {
        match self {
            Self::Decimal(value) => Ok(*value),
            _ => panic!("value is not a decimal"),
        }
    }

    pub fn as_boolean(&self) -> Result<bool, Error> {
        match self {
            Self::Boolean(value) => Ok(*value),
            _ => panic!("value is not a decimal"),
        }
    }

    #[rustfmt::skip]
    pub fn kind(&self) -> ExpressionKind {
        match self {
            Self::String(_)     => ExpressionKind::String,
            Self::Integer(_)    => ExpressionKind::Integer,
            Self::Decimal(_)    => ExpressionKind::Decimal,
            Self::Boolean(_)    => ExpressionKind::Boolean,
            Self::Structure(_)  => ExpressionKind::Structure,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionValue {
    Path(Path),
    String(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    Structure(StructureD),
}

impl Display for ExpressionValue {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(_)          => formatter.write_str("Path"),
            Self::String(value)    => formatter.write_str(&value.to_string()),
            Self::Integer(value)   => formatter.write_str(&value.to_string()),
            Self::Decimal(value)   => formatter.write_str(&value.to_string()),
            Self::Boolean(value)   => formatter.write_str(&value.to_string()),
            Self::Structure(value) => todo!(),
        }
    }
}

impl ExpressionValue {
    fn from_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        if let Some(token) = token_buffer.peek_value() {
            match token.class {
                TokenClass::Identifier(_value) => {
                    if let Some(token) = token_buffer.peek_ahead(1)
                        && token.class.kind() == TokenKind::CurlyBegin
                    {
                        return Ok(Self::Structure(StructureD::parse_token(token_buffer)?));
                    }

                    Ok(Self::Path(Path::parse_token(token_buffer)?))
                }
                TokenClass::String(value) => {
                    token_buffer.next();
                    Ok(Self::String(value))
                }
                TokenClass::Integer(value) => {
                    token_buffer.next();
                    Ok(Self::Integer(value))
                }
                TokenClass::Decimal(value) => {
                    token_buffer.next();
                    Ok(Self::Decimal(value))
                }
                TokenClass::Boolean(value) => {
                    token_buffer.next();
                    Ok(Self::Boolean(value))
                }
                _ => panic!(
                    "Alicia internal error: ExpressionValue::parse_token(): want_token() gave back a token that is not a possible value"
                ),
            }
        } else {
            panic!("TO-DO from_token")
        }
    }

    pub fn as_string(&self) -> Result<String, Error> {
        match self {
            Self::String(value) => Ok(value.to_string()),
            _ => panic!("value is not a decimal"),
        }
    }

    pub fn as_integer(&self) -> Result<i64, Error> {
        match self {
            Self::Integer(value) => Ok(*value),
            _ => panic!("value is not an integer"),
        }
    }

    pub fn as_decimal(&self) -> Result<f64, Error> {
        match self {
            Self::Decimal(value) => Ok(*value),
            _ => panic!("value is not a decimal"),
        }
    }

    pub fn as_boolean(&self) -> Result<bool, Error> {
        match self {
            Self::Boolean(value) => Ok(*value),
            _ => panic!("value is not a decimal"),
        }
    }

    #[rustfmt::skip]
    pub fn kind(&self) -> ExpressionKind {
        match self {
            Self::Path(_)       => ExpressionKind::Path,
            Self::String(_)     => ExpressionKind::String,
            Self::Integer(_)    => ExpressionKind::Integer,
            Self::Decimal(_)    => ExpressionKind::Decimal,
            Self::Boolean(_)    => ExpressionKind::Boolean,
            Self::Structure(_)  => ExpressionKind::Structure,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
}

impl ExpressionOperator {
    #[rustfmt::skip]
    fn from_token(token: Token) -> Self {
        match token.class.kind() {
            TokenKind::Add      => Self::Add,
            TokenKind::Subtract => Self::Subtract,
            TokenKind::Multiply => Self::Multiply,
            TokenKind::Divide   => Self::Divide,
            TokenKind::Equal    => Self::Equal,
            _ => panic!(
                "Alicia internal error: ExpressionValue::parse_token(): want_token() gave back a token that is not a possible value"
            ),
        }
    }

    #[rustfmt::skip]
    fn parse_token(&self, token_a: Expression, token_b: Expression) -> Expression {
        let token_a = Box::new(token_a);
        let token_b = Box::new(token_b);

        match self {
            Self::Add      => Expression::Operation(Self::Add,      token_a, token_b),
            Self::Subtract => Expression::Operation(Self::Subtract, token_a, token_b),
            Self::Multiply => Expression::Operation(Self::Multiply, token_a, token_b),
            Self::Divide   => Expression::Operation(Self::Divide,   token_a, token_b),
            Self::Equal    => Expression::Operation(Self::Equal,    token_a, token_b),
        }
    }

    #[rustfmt::skip]
    fn bind_power(&self) -> (f32, f32) {
        match self {
            Self::Add      => (1.0, 1.1),
            Self::Subtract => (1.0, 1.1),
            Self::Multiply => (2.0, 2.1),
            Self::Divide   => (2.0, 2.1),
            Self::Equal    => (1.0, 1.1),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Value(ExpressionValue),
    Operation(ExpressionOperator, Box<Expression>, Box<Expression>),
}

impl Expression {
    pub fn parse_token(token_buffer: &mut TokenBuffer, bind_power: f32) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Expression, |token_buffer| {
            let mut value_a = if token_buffer.want_peek(TokenKind::ParenthesisBegin) {
                token_buffer.want(TokenKind::ParenthesisBegin)?;
                let value = Self::parse_token(token_buffer, 0.0)?;
                token_buffer.want(TokenKind::ParenthesisClose)?;

                value
            } else {
                Expression::Value(ExpressionValue::from_token(token_buffer)?)
            };

            while let Some(token) = token_buffer.peek_operator() {
                let operator = ExpressionOperator::from_token(token);

                if operator.bind_power().0 <= bind_power {
                    break;
                }

                token_buffer.want_operator()?;

                let value_b = Self::parse_token(token_buffer, operator.bind_power().1)?;

                value_a = ExpressionOperator::parse_token(&operator, value_a, value_b)
            }

            Ok(value_a)
        })
    }

    pub fn evaluate(&self, scope: &Scope) -> Result<Option<Value>, Error> {
        Ok(match self {
            Expression::Value(value) => match value {
                ExpressionValue::Path(path) => {
                    let mut current = None;

                    for entry in &path.list {
                        match entry {
                            PathKind::Identifier(identifier) => {
                                if let Some(value) = scope.get_declaration(identifier.clone()) {
                                    match value {
                                        Declaration::Value(value) => {
                                            current = Some(value.clone());
                                        }
                                        _ => todo!(),
                                    }
                                } else {
                                    if let Some(current_p) = &mut current {
                                        match current_p {
                                            Value::Structure(structure) => {
                                                if let Some(field) =
                                                    structure.data.get(&identifier.text)
                                                {
                                                    current = Some(field.clone());
                                                } else {
                                                    panic!("unknown field: {identifier}")
                                                }
                                            }
                                            _ => todo!(),
                                        }
                                    } else {
                                        panic!("{entry:?}")
                                    }
                                }
                            }
                            PathKind::Invocation(invocation) => {
                                return Ok(invocation.execute(scope)?);
                            }
                            _ => todo!(),
                        }
                    }

                    current
                }
                ExpressionValue::String(value) => Some(Value::String(value.to_string())),
                ExpressionValue::Integer(value) => Some(Value::Integer(*value)),
                ExpressionValue::Decimal(value) => Some(Value::Decimal(*value)),
                ExpressionValue::Boolean(value) => Some(Value::Boolean(*value)),
                ExpressionValue::Structure(value) => {
                    Some(Value::Structure(StructureV::new(value.clone(), scope)?))
                }
            },
            Expression::Operation(operator, a, b) => {
                let a = a.evaluate(scope)?;
                let b = b.evaluate(scope)?;

                if let Some(a) = a
                    && let Some(b) = b
                {
                    let kind_a = a.kind();
                    let kind_b = b.kind();

                    if kind_a == kind_b {
                        match kind_a {
                            ExpressionKind::Path => todo!(),
                            ExpressionKind::String => todo!(),
                            ExpressionKind::Integer => {
                                let a = a.as_integer()?;
                                let b = b.as_integer()?;

                                Some(match operator {
                                    ExpressionOperator::Add => Value::Integer(a + b),
                                    ExpressionOperator::Subtract => Value::Integer(a - b),
                                    ExpressionOperator::Multiply => Value::Integer(a * b),
                                    ExpressionOperator::Divide => Value::Integer(a / b),
                                    ExpressionOperator::Equal => Value::Boolean(a == b),
                                })
                            }
                            ExpressionKind::Decimal => {
                                let a = a.as_decimal()?;
                                let b = b.as_decimal()?;

                                Some(match operator {
                                    ExpressionOperator::Add => Value::Decimal(a + b),
                                    ExpressionOperator::Subtract => Value::Decimal(a - b),
                                    ExpressionOperator::Multiply => Value::Decimal(a * b),
                                    ExpressionOperator::Divide => Value::Decimal(a / b),
                                    ExpressionOperator::Equal => Value::Boolean(a == b),
                                })
                            }
                            ExpressionKind::Boolean => todo!(),
                            ExpressionKind::Structure => todo!(),
                        }
                    } else {
                        panic!("evaluate: a is null, or b is null");
                    }
                } else {
                    panic!("evaluate: type mismatch");
                }
            }
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        // TO-DO soft evaluation where we analyze if a variable is or isn't in scope,
        // do type-checking, etc.

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub span: TokenSpan,
    pub name: Identifier,
    pub kind: Identifier,
    pub value: Expression,
}

impl Definition {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Definition, |token_buffer| {
            token_buffer.want(TokenKind::Let)?;
            let name = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::Colon)?;
            let kind = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::Definition)?;
            let value = Expression::parse_token(token_buffer, 0.0)?;

            token_buffer.want(TokenKind::ColonSemi)?;

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

        self.value.analyze(scope)?;

        Ok(())
    }

    pub fn execute(&self, scope: &mut Scope) -> Result<(), Error> {
        let value = self.value.evaluate(scope)?;

        if let Some(value) = value {
            scope.set_declaration(self.name.clone(), Declaration::Value(value));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub span: TokenSpan,
    pub name: Identifier,
    pub kind: Token,
    pub value: Expression,
}

impl Assignment {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Assignment, |token_buffer| {
            let name = token_buffer.want_identifier()?;
            let kind = token_buffer.want_definition()?;
            let value = Expression::parse_token(token_buffer, 0.0)?;
            token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self {
                span: token_buffer.get_span(),
                name,
                kind,
                value,
            })
        })
    }

    pub fn parse_token_loose(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Assignment, |token_buffer| {
            let name = token_buffer.want_identifier()?;
            let kind = token_buffer.want_definition()?;
            let value = Expression::parse_token(token_buffer, 0.0)?;

            Ok(Self {
                span: token_buffer.get_span(),
                name,
                kind,
                value,
            })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        if let Some(variable) = scope.get_declaration(self.name.clone()) {
            match variable {
                Declaration::Definition(_) => {
                    Ok(())
                    // TO-DO type check that whatever we're assigning to the
                    // definition is valid.
                }
                _ => Err(Error::new_info(
                    ErrorInfo::new_point(self.span.clone(), Some(self.name.point)),
                    ErrorKind::InvalidAssignment(self.name.clone()),
                    Some(ErrorHint::Assignment),
                )),
            }
        } else {
            Err(Error::new_info(
                ErrorInfo::new_point(self.span.clone(), Some(self.name.point)),
                ErrorKind::UnknownSymbol(self.name.clone()),
                Some(ErrorHint::Assignment),
            ))
        }?;

        self.value.analyze(scope)?;

        Ok(())
    }

    pub fn execute(&self, scope: &mut Scope) -> Result<(), Error> {
        let value = self.value.evaluate(scope)?;

        if let Some(value) = value {
            scope.set_declaration(self.name.clone(), Declaration::Value(value));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Invocation {
    pub span: TokenSpan,
    pub name: Identifier,
    pub list: Vec<Expression>,
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

                list.push(Expression::parse_token(token_buffer, 0.0)?);

                if let Some(token) = token_buffer.peek()
                    && token.class.kind() == TokenKind::Comma
                {
                    token_buffer.next();
                } else {
                    break;
                }
            }

            token_buffer.want(TokenKind::ParenthesisClose)?;

            token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self {
                span: token_buffer.get_span(),
                name,
                list,
            })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        if let Some(declaration) = scope.get_declaration(self.name.clone()) {
            match declaration {
                Declaration::Function(_) => {
                    // TO-DO type check and validate the function is receiving every argument
                    for expression in &self.list {
                        expression.analyze(scope)?;
                    }

                    Ok(())
                }
                Declaration::FunctionNative(_) => {
                    // TO-DO type check and validate the function is receiving every argument
                    for expression in &self.list {
                        expression.analyze(scope)?;
                    }

                    Ok(())
                }
                _ => Err(Error::new_info(
                    ErrorInfo::new_token(self.span.clone(), None),
                    ErrorKind::InvalidInvocation(self.name.clone()),
                    Some(ErrorHint::Invocation),
                )),
            }
        } else {
            Err(Error::new_info(
                ErrorInfo::new_token(self.span.clone(), None),
                ErrorKind::UnknownSymbol(self.name.clone()),
                Some(ErrorHint::Invocation),
            ))
        }
    }

    pub fn execute(&self, scope: &Scope) -> Result<Option<Value>, Error> {
        if let Some(declaration) = scope.get_declaration(self.name.clone()) {
            match declaration {
                Declaration::Function(function) => {
                    // TO-DO type check and validate the function is receiving every argument
                    for expression in &self.list {
                        expression.analyze(scope)?;
                    }

                    function.execute(scope)
                }
                Declaration::FunctionNative(function) => {
                    // TO-DO type check and validate the function is receiving every argument
                    for expression in &self.list {
                        expression.analyze(scope)?;
                    }

                    (function.call)(ArgumentBuffer::new(self.list.clone(), scope)?)
                }
                _ => Err(Error::new_info(
                    ErrorInfo::new_token(self.span.clone(), None),
                    ErrorKind::InvalidInvocation(self.name.clone()),
                    Some(ErrorHint::Invocation),
                )),
            }
        } else {
            Err(Error::new_info(
                ErrorInfo::new_token(self.span.clone(), None),
                ErrorKind::UnknownSymbol(self.name.clone()),
                Some(ErrorHint::Invocation),
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Indexation {
    pub name: Identifier,
    pub expression: Expression,
}

impl Indexation {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Invocation, |token_buffer| {
            let name = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::SquareBegin)?;

            let expression = Expression::parse_token(token_buffer, 0.0)?;

            token_buffer.want(TokenKind::SquareClose)?;

            token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self { name, expression })
        })
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

    pub fn analyze(&self, scope: &Scope, iteration: bool) -> Result<(), Error> {
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
                Instruction::Condition(condition) => condition.analyze(&scope_block)?,
                Instruction::Iteration(iteration) => iteration.analyze(&scope_block)?,
                Instruction::Block(block) => block.analyze(&scope_block, false)?,
                Instruction::Skip => {
                    if !iteration {
                        // TO-DO use actual span data.
                        return Err(Error::new_kind(
                            ErrorKind::InvalidSkip,
                            Some(ErrorHint::Iteration),
                        ));
                    }
                }
                Instruction::Exit => {
                    if !iteration {
                        // TO-DO use actual span data.
                        return Err(Error::new_kind(
                            ErrorKind::InvalidExit,
                            Some(ErrorHint::Iteration),
                        ));
                    }
                }
                /*
                Instruction::Return(_) => todo!(),
                */
                _ => {}
            }
        }

        Ok(())
    }

    pub fn execute(&self, scope: &Scope) -> Result<Option<Value>, Error> {
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

        // TO-DO only return from this code block on the return instruction.
        for instruction in &self.code {
            match instruction {
                Instruction::Definition(definition) => {
                    definition.execute(&mut scope_block)?;
                }
                Instruction::Assignment(assignment) => {
                    assignment.execute(&mut scope_block)?;
                }
                Instruction::Invocation(invocation) => {
                    invocation.execute(&scope_block)?;
                }
                Instruction::Condition(condition) => condition.execute(&scope_block)?,
                Instruction::Iteration(iteration) => iteration.execute(&scope_block)?,
                Instruction::Block(block) => {
                    block.execute(&scope_block)?;
                }
                Instruction::Skip => todo!(),
                Instruction::Exit => todo!(),
                Instruction::Return(result) => {
                    if let Some(value) = &result.value {
                        return value.evaluate(&scope_block);
                    } else {
                        return Ok(None);
                    }
                }
                _ => todo!(),
            };
        }

        Ok(None)
    }

    pub fn execute_loop(&self, scope: &Scope) -> Result<(Option<Value>, Option<bool>), Error> {
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

        // TO-DO only return from this code block on the return instruction.
        for instruction in &self.code {
            match instruction {
                Instruction::Definition(definition) => {
                    definition.execute(&mut scope_block)?;
                }
                Instruction::Assignment(assignment) => {
                    assignment.execute(&mut scope_block)?;
                }
                Instruction::Invocation(invocation) => {
                    invocation.execute(&scope_block)?;
                }
                Instruction::Condition(condition) => condition.execute(&scope_block)?,
                Instruction::Iteration(iteration) => iteration.execute(&scope_block)?,
                Instruction::Block(block) => {
                    block.execute(&scope_block)?;
                }
                Instruction::Skip => {
                    return Ok((None, Some(false)));
                }
                Instruction::Exit => {
                    return Ok((None, Some(true)));
                }
                Instruction::Return(result) => {
                    if let Some(value) = &result.value {
                        return Ok((value.evaluate(&scope_block)?, None));
                    } else {
                        return Ok((None, None));
                    }
                }
                _ => todo!(),
            };
        }

        Ok((None, None))
    }
}

pub type FunctionSignature = fn(ArgumentBuffer) -> Result<Option<Value>, Error>;

#[derive(Debug, Clone)]
pub struct FunctionNative {
    pub call: FunctionSignature,
    pub enter: Vec<ExpressionKind>,
    pub leave: Option<ExpressionKind>,
}

impl FunctionNative {
    pub fn new(
        call: FunctionSignature,
        enter: Vec<ExpressionKind>,
        leave: Option<ExpressionKind>,
    ) -> Self {
        Self { call, enter, leave }
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
                Instruction::parse_comma(
                    token_buffer,
                    TokenKind::ParenthesisClose,
                    |token_buffer| {
                        enter.push(Variable::parse_token(token_buffer)?);
                        Ok(())
                    },
                )?;

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

        self.block.analyze(scope, false)?;

        Ok(())
    }

    pub fn execute(&self, scope: &Scope) -> Result<Option<Value>, Error> {
        self.block.execute(scope)
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
                    ErrorInfo::new_point(span.clone(), Some(name.point)),
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
pub struct StructureD {
    pub name: Identifier,
    pub list: Vec<Assignment>,
}

impl StructureD {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Structure, |token_buffer| {
            let mut list = Vec::new();

            let name = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::CurlyBegin)?;

            Instruction::parse_comma(token_buffer, TokenKind::CurlyClose, |token_buffer| {
                list.push(Assignment::parse_token_loose(token_buffer)?);
                Ok(())
            })?;

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self { name, list })
        })
    }
}

#[derive(Debug, Clone)]
pub struct StructureV {
    pub name: Identifier,
    pub data: HashMap<String, Value>,
}

impl Display for StructureV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{} ", self.name))?;

        f.write_str("{\n")?;

        for (k, v) in &self.data {
            f.write_str(&format!("  {k}: {v},\n"))?;
        }

        f.write_str("}")?;

        Ok(())
    }
}

impl StructureV {
    pub fn new(structure: StructureD, scope: &Scope) -> Result<Self, Error> {
        let name = structure.name.clone();
        let mut data = HashMap::default();

        // TO-DO actual type checking
        for assignment in structure.list {
            if let Some(value) = assignment.value.evaluate(scope)? {
                data.insert(assignment.name.text, value);
            }
        }

        Ok(Self { name, data })
    }
}

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

            Instruction::parse_comma(token_buffer, TokenKind::CurlyClose, |token_buffer| {
                list.push(Variable::parse_token(token_buffer)?);
                Ok(())
            })?;

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

            Instruction::parse_comma(token_buffer, TokenKind::CurlyClose, |token_buffer| {
                list.push(token_buffer.want_identifier()?);
                Ok(())
            })?;

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self { name, list })
        })
    }
}

//================================================================

#[derive(Debug, Clone)]
pub enum PathKind {
    Identifier(Identifier),
    Invocation(Invocation),
    Indexation(Indexation),
}

#[derive(Debug, Clone)]
pub struct Path {
    list: Vec<PathKind>,
}

impl Path {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Use, |token_buffer| {
            let mut list = Vec::new();

            while let Some(token) = token_buffer.peek() {
                if token.class.kind() == TokenKind::Identifier {
                    if let Some(token) = token_buffer.peek_ahead(1) {
                        if token.class.kind() == TokenKind::ParenthesisBegin {
                            list.push(PathKind::Invocation(Invocation::parse_token(token_buffer)?));
                            continue;
                        } else if token.class.kind() == TokenKind::SquareBegin {
                            list.push(PathKind::Indexation(Indexation::parse_token(token_buffer)?));
                            continue;
                        }
                    }

                    list.push(PathKind::Identifier(token_buffer.want_identifier()?))
                } else if token.class.kind() == TokenKind::Dot {
                    token_buffer.want(TokenKind::Dot)?;
                } else {
                    break;
                }
            }

            Ok(Self { list })
        })
    }
}

/*
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
*/
