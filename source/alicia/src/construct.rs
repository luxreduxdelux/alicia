use crate::buffer::*;
use crate::error::*;
use crate::helper::*;
use crate::machine::Function as MFunction;
use crate::machine::Instruction;
use crate::machine::Value;
use crate::scope::*;
use crate::token::*;

//================================================================

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

//================================================================

/*
iteration compilation:
    let a := [1, 2, 3];
    let i := 0;
    let l := a.length();

    loop (i < l) {
        i := i + 1;
        let x := a[i - 1];
    }

    <->

    let a := [1, 2, 3];

    loop (x := a) {
        print("{}", x);
    }
*/

#[derive(Debug, Clone)]
pub enum Statement {
    Function(Function),
    Structure(Structure),
    Enumerate(Enumerate),
    Definition(Definition),
    Assignment(Assignment),
    Expression(Expression),
    Condition(Condition),
    Iteration(Iteration),
    Block(Block),
    Skip,
    Exit,
    Return(Return),
}

impl Statement {
    fn parse_identifier(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        let expression = Expression::parse_token(token_buffer, 0.0)?;

        if token_buffer.want_peek(TokenKind::ColonSemi) {
            token_buffer.want(TokenKind::ColonSemi)?;
            return Ok(Self::Expression(expression));
        }

        Ok(Self::Assignment(Assignment::parse_token(
            token_buffer,
            expression,
        )?))
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

    #[rustfmt::skip]
    pub fn parse_token(token: Token, token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        match token.class {
            TokenClass::Function  => Ok(Self::Function(Function::parse_token(token_buffer, None)?)),
            TokenClass::Structure => Ok(Self::Structure(Structure::parse_token(token_buffer)?)),
            TokenClass::Enumerate => Ok(Self::Enumerate(Enumerate::parse_token(token_buffer)?)),
            TokenClass::Let       => Ok(Self::Definition(Definition::parse_token(token_buffer)?)),
            TokenClass::If        => Ok(Self::Condition(Condition::parse_token(
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
            TokenClass::Return        => Ok(Self::Return(Return::parse_token(token_buffer)?)),
            TokenClass::Identifier(_) => Ok(Self::parse_identifier(token_buffer)?),
            TokenClass::CurlyBegin    => Ok(Self::Block(Block::parse_token(token_buffer)?)),
            _ => Error::new_info(
                token_buffer.get_error_info(Some(token.clone())),
                ErrorKind::UnknownToken(token),
                Some(ErrorHint::Function),
            ),
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
                if token_buffer.want_peek(TokenKind::ParenthesisBegin) {
                    token_buffer.want(TokenKind::ParenthesisBegin)?;
                    let value = Expression::parse_token(token_buffer, 0.0)?;
                    token_buffer.want(TokenKind::ParenthesisClose)?;

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

                token_buffer.want(TokenKind::ParenthesisBegin)?;
                let value = Expression::parse_token(token_buffer, 0.0)?;
                token_buffer.want(TokenKind::ParenthesisClose)?;

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

    pub fn analyze(&mut self, scope: ScopePointer) -> Result<(), Error> {
        if let Some(value) = &self.value {
            let kind = value.analyze(&mut scope.borrow_mut(), None)?;

            if kind != ExpressionKind::Boolean {
                panic!("condition expression kind is not a boolean");
            }
        }

        if let Some(child) = &mut self.child {
            child.analyze(scope.clone())?;
        }

        self.block.analyze(scope, Vec::default(), false)?;

        Ok(())
    }

    fn analyze_flow(&self, scope: &Scope) -> Result<Vec<Flow>, Error> {
        let mut list = Vec::new();

        list.push(self.block.analyze_flow(scope, self.value.is_some())?);

        if let Some(child) = &self.child {
            list.extend(child.analyze_flow(scope)?);
        }

        Ok(list)
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        let head = function.cursor();

        let branch = if let Some(value) = &self.value {
            value.compile(scope, function)?;

            let branch = function.cursor();

            function.push(Instruction::Null);

            Some(branch)
        } else {
            None
        };

        self.block.compile(scope, function, true, None)?;

        let jump = function.cursor();

        function.push(Instruction::Null);

        let tail = function.cursor();

        if let Some(branch) = branch {
            function.change(Instruction::Branch(tail - 1), branch);
        }

        if let Some(child) = &self.child {
            child.compile(scope, function)?;
        }

        function.change(Instruction::Jump(function.cursor()), jump);

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

    pub fn analyze(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
        if let Some(value) = &self.value {
            value.analyze(scope, None)
        } else {
            Ok(ExpressionKind::Null)
        }
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        if let Some(value) = &self.value {
            value.compile(scope, function)?;
            function.push(Instruction::Return(true));
        } else {
            function.push(Instruction::Return(false));
        }

        Ok(())
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

            let value = if token_buffer.want_peek(TokenKind::ParenthesisBegin) {
                //if let Some(token) = token_buffer.peek_ahead(1)
                //    && token.class.kind() == TokenKind::Definition
                //{
                //    Some(IterationValue::Iterational(Assignment::parse_token(
                //        token_buffer,
                //    )?))
                //} else {

                token_buffer.want(TokenKind::ParenthesisBegin)?;
                let value = Expression::parse_token(token_buffer, 0.0)?;
                token_buffer.want(TokenKind::ParenthesisClose)?;

                Some(IterationValue::Conditional(value))
                //}
            } else {
                None
            };

            let block = Block::parse_token(token_buffer)?;

            Ok(Self { value, block })
        })
    }

    pub fn analyze(&mut self, scope: ScopePointer) -> Result<(), Error> {
        if let Some(value) = &self.value {
            match value {
                IterationValue::Iterational(assignment) => assignment.analyze(&scope.borrow())?,
                IterationValue::Conditional(expression) => {
                    expression.analyze(&scope.borrow(), None)?;
                }
            };
        }

        self.block.analyze(scope, Vec::default(), true)?;

        Ok(())
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        let cursor = function.cursor();
        let mut branch = None;

        if let Some(value) = &self.value {
            match value {
                IterationValue::Iterational(assignment) => todo!(),
                IterationValue::Conditional(expression) => {
                    expression.compile(scope, function)?;

                    branch = Some(function.cursor());

                    function.push(Instruction::Null);
                }
            }
        }

        self.block.compile(scope, function, false, Some(cursor))?;
        function.push(Instruction::Jump(cursor));

        if let Some(branch) = branch {
            function.change(Instruction::Branch(function.cursor() - 1), branch);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Eq)]
pub enum ExpressionKind {
    Null,
    Identifier,
    String,
    Integer,
    Decimal,
    Boolean,
    Function(Identifier),
    FunctionNative(Identifier),
    DeclarationStructure(Identifier),
    DeclarationEnumerate(Identifier),
    Structure(Identifier),
    Enumerate(Identifier),
    Array(Box<ExpressionKind>),
    Table(Box<ExpressionKind>, Box<ExpressionKind>),
    //Tuple(Vec<ExpressionKind>),
}

impl PartialEq for ExpressionKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Structure(l0), Self::Structure(r0)) => l0.text == r0.text,
            (Self::Enumerate(l0), Self::Enumerate(r0)) => l0.text == r0.text,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl ExpressionKind {
    fn is_number(&self) -> bool {
        *self == Self::Integer || *self == Self::Decimal
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionValue {
    Null,
    Identifier(Identifier),
    String(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    Structure(StructureD),
    Enumerate(EnumerateD),
    Array(ArrayD),
    Table(TableD),
}

impl Display for ExpressionValue {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null              => formatter.write_str("Null"),
            Self::Identifier(value) => formatter.write_str(&value.text),
            Self::String(value)     => formatter.write_str(&value.to_string()),
            Self::Integer(value)    => formatter.write_str(&value.to_string()),
            Self::Decimal(value)    => formatter.write_str(&value.to_string()),
            Self::Boolean(value)    => formatter.write_str(&value.to_string()),
            _ => todo!(),
        }
    }
}

impl ExpressionValue {
    fn from_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        if let Some(token) = token_buffer.peek_value() {
            match token.class {
                TokenClass::Identifier(_) => {
                    if let Some(token) = token_buffer.peek_ahead(1)
                        && token.class.kind() == TokenKind::CurlyBegin
                    {
                        return Ok(Self::Structure(StructureD::parse_token(token_buffer)?));
                    }

                    if let Some(token) = token_buffer.peek_ahead(3)
                        && token.class.kind() == TokenKind::CurlyBegin
                    {
                        return Ok(Self::Enumerate(EnumerateD::parse_token(token_buffer)?));
                    }

                    Ok(Self::Identifier(token_buffer.want_identifier()?))
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
                TokenClass::SquareBegin => Ok(Self::Array(ArrayD::parse_token(token_buffer)?)),
                TokenClass::CurlyBegin => Ok(Self::Table(TableD::parse_token(token_buffer)?)),
                _ => panic!(
                    "Alicia internal error: ExpressionValue::parse_token(): want_token() gave back a token that is not a possible value"
                ),
            }
        } else {
            panic!("empty token buffer in from_token")
        }
    }

    #[rustfmt::skip]
    pub fn kind(&self, scope: &Scope, infer: Option<ExpressionKind>) -> Result<ExpressionKind, Error> {
        Ok(match self {
            Self::Null          => ExpressionKind::Null,
            Self::Identifier(_) => ExpressionKind::Identifier,
            Self::String(_)     => ExpressionKind::String,
            Self::Integer(_)    => ExpressionKind::Integer,
            Self::Decimal(_)    => ExpressionKind::Decimal,
            Self::Boolean(_)    => ExpressionKind::Boolean,
            Self::Structure(x)  => ExpressionKind::Structure(x.name.clone()),
            Self::Enumerate(x)  => ExpressionKind::Enumerate(x.name.clone()),
            Self::Array(x)      => x.analyze(scope, infer)?,
            Self::Table(x)      => x.analyze(scope, infer)?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionOperator {
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
    Dot,
    Reference,
    Invocation(Vec<Expression>),
    Indexation(Option<Box<Expression>>),
}

impl ExpressionOperator {
    #[rustfmt::skip]
    fn from_token(token: Token) -> Self {
        match token.class.kind() {
            TokenKind::Add              => Self::Add,
            TokenKind::Subtract         => Self::Subtract,
            TokenKind::Multiply         => Self::Multiply,
            TokenKind::Divide           => Self::Divide,
            TokenKind::Not              => Self::Not,
            TokenKind::And              => Self::And,
            TokenKind::Or               => Self::Or,
            TokenKind::GT               => Self::GT,
            TokenKind::LT               => Self::LT,
            TokenKind::Equal            => Self::Equal,
            TokenKind::GTE              => Self::GTE,
            TokenKind::LTE              => Self::LTE,
            TokenKind::EqualNot         => Self::EqualNot,
            TokenKind::Dot              => Self::Dot,
            TokenKind::Ampersand        => Self::Reference,
            TokenKind::ParenthesisBegin => Self::Invocation(Vec::default()),
            TokenKind::SquareBegin      => Self::Indexation(None),
            _ => panic!(
                "Alicia internal error: ExpressionValue::parse_token(): want_token() gave back a token that is not a possible value"
            ),
        }
    }

    #[rustfmt::skip]
    fn parse_token_mono(&self, token_a: Expression) -> ExpressionData {
        let token_a = Box::new(token_a);

        match self {
            Self::Subtract      => ExpressionData::OperationPrior(Self::Subtract,   token_a),
            Self::Reference     => ExpressionData::OperationPrior(Self::Reference,  token_a),
            Self::Invocation(_) => ExpressionData::OperationAfter(token_a, self.clone()),
            Self::Indexation(_) => ExpressionData::OperationAfter(token_a, self.clone()),
            x => panic!("incorrect parse_token_mono operator: {x:?}")
        }
    }

    #[rustfmt::skip]
    fn parse_token_binary(&self, token_a: Expression, token_b: Expression) -> ExpressionData {
        let token_a = Box::new(token_a);
        let token_b = Box::new(token_b);

        match self {
            Self::Add      => ExpressionData::Operation(Self::Add,      token_a, token_b),
            Self::Subtract => ExpressionData::Operation(Self::Subtract, token_a, token_b),
            Self::Multiply => ExpressionData::Operation(Self::Multiply, token_a, token_b),
            Self::Divide   => ExpressionData::Operation(Self::Divide,   token_a, token_b),
            Self::And      => ExpressionData::Operation(Self::And,      token_a, token_b),
            Self::Or       => ExpressionData::Operation(Self::Or,       token_a, token_b),
            Self::GT       => ExpressionData::Operation(Self::GT,       token_a, token_b),
            Self::LT       => ExpressionData::Operation(Self::LT,       token_a, token_b),
            Self::Equal    => ExpressionData::Operation(Self::Equal,    token_a, token_b),
            Self::GTE      => ExpressionData::Operation(Self::GTE,      token_a, token_b),
            Self::LTE      => ExpressionData::Operation(Self::LTE,      token_a, token_b),
            Self::EqualNot => ExpressionData::Operation(Self::EqualNot, token_a, token_b),
            Self::Dot      => ExpressionData::Operation(Self::Dot, token_a, token_b),
            x => panic!("incorrect parse_token_binary operator: {x:?}")
        }
    }

    #[rustfmt::skip]
    fn bind_power(&self) -> (f32, f32) {
        match self {
            Self::Add           => (1.0, 1.1),
            Self::Subtract      => (1.0, 1.1),
            Self::Multiply      => (2.0, 2.1),
            Self::Divide        => (2.0, 2.1),
            // TO-DO add actual bind power to these
            Self::Not           => (1.0, 1.1),
            Self::And           => (1.0, 1.1),
            Self::Or            => (1.0, 1.1),
            Self::GT            => (1.0, 1.1),
            Self::LT            => (1.0, 1.1),
            Self::Equal         => (1.0, 1.1),
            Self::GTE           => (1.0, 1.1),
            Self::LTE           => (1.0, 1.1),
            Self::EqualNot      => (1.0, 1.1),
            Self::Dot           => (1.0, 1.1),
            Self::Reference     => (2.0, 2.1),
            Self::Invocation(_) => (2.1, 2.0),
            Self::Indexation(_) => (2.1, 2.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub span: TokenSpan,
    pub data: ExpressionData,
}

impl Expression {
    fn new(token_buffer: &mut TokenBuffer, data: ExpressionData) -> Self {
        Self {
            span: token_buffer.get_span(),
            data,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionData {
    Value(ExpressionValue),
    Operation(ExpressionOperator, Box<Expression>, Box<Expression>),
    OperationPrior(ExpressionOperator, Box<Expression>),
    OperationAfter(Box<Expression>, ExpressionOperator),
}

impl Expression {
    pub fn parse_token(token_buffer: &mut TokenBuffer, bind_power: f32) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Expression, |token_buffer| {
            let mut value_a = if token_buffer.want_peek(TokenKind::ParenthesisBegin) {
                token_buffer.want(TokenKind::ParenthesisBegin)?;
                let value = Self::parse_token(token_buffer, 0.0)?;
                token_buffer.want(TokenKind::ParenthesisClose)?;

                value
            } else if token_buffer.want_peek(TokenKind::SquareBegin) {
                let value = ExpressionData::Value(ExpressionValue::from_token(token_buffer)?);

                Expression::new(token_buffer, value)
            } else if token_buffer.peek_operator().is_some() {
                let operator = ExpressionOperator::from_token(token_buffer.want_operator()?);
                let value = Self::parse_token(token_buffer, 0.0)?;

                Expression::new(
                    token_buffer,
                    ExpressionOperator::parse_token_mono(&operator, value),
                )
            } else {
                let value = ExpressionData::Value(ExpressionValue::from_token(token_buffer)?);
                let value = Expression::new(token_buffer, value);
                token_buffer.push_span();

                if let Some(operator) = token_buffer.peek_operator() {
                    match operator.class.kind() {
                        TokenKind::ParenthesisBegin => {
                            let mut operator =
                                ExpressionOperator::from_token(token_buffer.want_operator()?);

                            match &mut operator {
                                ExpressionOperator::Invocation(list) => {
                                    Statement::parse_comma(
                                        token_buffer,
                                        TokenKind::ParenthesisClose,
                                        |token_buffer| {
                                            list.push(Expression::parse_token(token_buffer, 0.0)?);
                                            Ok(())
                                        },
                                    )?;

                                    token_buffer.want(TokenKind::ParenthesisClose)?;

                                    Expression::new(
                                        token_buffer,
                                        ExpressionOperator::parse_token_mono(&operator, value),
                                    )
                                }
                                _ => value,
                            }
                        }
                        TokenKind::SquareBegin => {
                            // TO-DO this is so fucking stupid, just *want* the SquareBegin and
                            // return an Indexation rather than getting it from from_token which is dumb
                            let mut operator =
                                ExpressionOperator::from_token(token_buffer.want_operator()?);

                            match &mut operator {
                                ExpressionOperator::Indexation(expression) => {
                                    *expression =
                                        Some(Box::new(Expression::parse_token(token_buffer, 0.0)?));

                                    token_buffer.want(TokenKind::SquareClose)?;

                                    Expression::new(
                                        token_buffer,
                                        ExpressionOperator::parse_token_mono(&operator, value),
                                    )
                                }
                                _ => value,
                            }
                        }
                        _ => value,
                    }
                } else {
                    value
                }
            };

            while let Some(token) = token_buffer.peek_operator() {
                let operator = ExpressionOperator::from_token(token);

                if operator.bind_power().0 <= bind_power {
                    break;
                }

                token_buffer.want_operator()?;

                let value_b = Self::parse_token(token_buffer, operator.bind_power().1)?;

                value_a = Expression::new(
                    token_buffer,
                    ExpressionOperator::parse_token_binary(&operator, value_a, value_b),
                )
            }

            Ok(value_a)
        })
    }

    pub fn analyze_identifier(&self) -> Result<Identifier, Error> {
        match &self.data {
            ExpressionData::Value(value) => match value {
                ExpressionValue::Identifier(identifier) => Ok(identifier.clone()),
                _ => panic!("analyze_identifier: value is not an identifier"),
            },
            ExpressionData::Operation(_, a, _) => a.analyze_identifier(),
            ExpressionData::OperationAfter(e, _) => e.analyze_identifier(),
            ExpressionData::OperationPrior(_, e) => e.analyze_identifier(),
        }
    }

    pub fn analyze(
        &self,
        scope: &Scope,
        infer: Option<ExpressionKind>,
    ) -> Result<ExpressionKind, Error> {
        match &self.data {
            ExpressionData::Value(value) => match value {
                ExpressionValue::Identifier(identifier) => {
                    let value = scope
                        .get_declaration(identifier.clone())
                        .expect(&format!("no declaration for identifier {identifier:?}"));

                    match value {
                        Declaration::Function(function) => {
                            Ok(ExpressionKind::Function(function.name.clone()))
                        }
                        Declaration::FunctionNative(function) => {
                            // TO-DO FNative's name should already be an Identifier...
                            Ok(ExpressionKind::FunctionNative(
                                Identifier::from_string(function.name.clone(), Point::default())
                                    .unwrap(),
                            ))
                        }
                        Declaration::Definition(definition) => {
                            if let Some(kind) = &definition.kind_e {
                                Ok(kind.clone())
                            } else {
                                definition.value.analyze(scope, infer)
                            }
                        }
                        Declaration::Structure(_) => {
                            Ok(ExpressionKind::DeclarationStructure(identifier.clone()))
                        }
                        Declaration::Enumerate(_) => {
                            Ok(ExpressionKind::DeclarationEnumerate(identifier.clone()))
                        }
                    }
                }
                ExpressionValue::Structure(structure_d) => structure_d.analyze(scope),
                //ExpressionValue::Enumerate(enumerate_d) => enumerate_d.analyze(scope),
                ExpressionValue::Array(array_d) => array_d.analyze(scope, infer),
                _ => value.kind(scope, infer),
            },
            ExpressionData::Operation(operator, e_a, e_b) => {
                let a = e_a.analyze(scope, infer.clone())?;

                if let ExpressionOperator::Dot = operator {
                    let b = e_b.analyze_identifier()?;

                    match &a {
                        ExpressionKind::Structure(identifier) => {
                            let structure = scope.get_structure(identifier.clone()).unwrap();

                            if let Some(field) = structure.variable.get(&b.text) {
                                return field.kind.type_check(scope);
                            }
                        }
                        ExpressionKind::DeclarationStructure(identifier) => {
                            let structure = scope.get_structure(identifier.clone()).unwrap();

                            if let Some(field) = structure.function.get(&b.text) {
                                if let Some(leave) = &field.leave {
                                    return leave.type_check(scope);
                                } else {
                                    return Ok(ExpressionKind::Null);
                                }
                            }
                        }
                        _ => panic!("dot operator: a is not a structure {a:?}"),
                    }
                }

                let b = e_b.analyze(scope, infer)?;

                if a != b {
                    // TO-DO add expression span.
                    //return Err(Error::new_info(
                    //    ErrorInfo::new_point(e_a.span.clone(), None),
                    //    ErrorKind::MixKind(a, b),
                    //    None,
                    //));
                    panic!("type mismatch: {:?} != {:?}", a, b);
                }

                if a.is_number() {
                    match operator {
                        ExpressionOperator::Add => Ok(a),
                        ExpressionOperator::Subtract => Ok(a),
                        ExpressionOperator::Multiply => Ok(a),
                        ExpressionOperator::Divide => Ok(a),
                        ExpressionOperator::GT => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::LT => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::Equal => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::GTE => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::LTE => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::EqualNot => Ok(ExpressionKind::Boolean),
                        _ => panic!("unsupported operator {operator:?} for value of type {a:?}"),
                    }
                } else if a == ExpressionKind::Boolean {
                    match operator {
                        ExpressionOperator::And => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::Or => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::Equal => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::EqualNot => Ok(ExpressionKind::Boolean),
                        _ => panic!("unsupported operator {operator:?} for value of type {a:?}"),
                    }
                } else {
                    match operator {
                        ExpressionOperator::Equal => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::EqualNot => Ok(ExpressionKind::Boolean),
                        _ => panic!("unsupported operator {operator:?} for value of type {a:?}"),
                    }
                }
            }
            ExpressionData::OperationPrior(operator, value) => {
                let value = value.analyze(scope, infer)?;

                if value.is_number() {
                    match operator {
                        ExpressionOperator::Subtract => Ok(value),
                        _ => {
                            panic!("unsupported operator {operator:?} for value of type {value:?}")
                        }
                    }
                } else if value == ExpressionKind::Boolean {
                    match operator {
                        ExpressionOperator::Not => Ok(ExpressionKind::Boolean),
                        _ => {
                            panic!("unsupported operator {operator:?} for value of type {value:?}")
                        }
                    }
                } else {
                    // TO-DO add reference
                    panic!("unsupported operator {operator:?} for value of type {value:?}")
                }
            }
            ExpressionData::OperationAfter(value_o, operator) => {
                let value = value_o.analyze(scope, infer.clone())?;

                match operator {
                    ExpressionOperator::Invocation(list) => {
                        match value {
                            ExpressionKind::Function(identifier) => {
                                let function = scope.get_function(identifier.clone()).unwrap();
                                let enter_a = function.enter.len();
                                let enter_b = list.len();

                                if enter_a != enter_b {
                                    return Error::new_info(
                                        ErrorInfo::new_point(
                                            self.span.clone(),
                                            self.span.begin,
                                            scope.get_active_source(),
                                        ),
                                        ErrorKind::InvalidInvocationArgumentLength(
                                            identifier, enter_b, enter_a,
                                        ),
                                        None,
                                    );
                                }

                                for (i, parameter) in function.enter.iter().enumerate() {
                                    let source = list[i].analyze(scope, infer.clone())?;
                                    let target = parameter.analyze(scope)?;

                                    if source != target {
                                        panic!(
                                            "function: argument type mis-match ({source:?} != {target:?})"
                                        );
                                    }
                                }

                                if let Some(leave) = &function.leave {
                                    leave.type_check(scope)
                                } else {
                                    Ok(ExpressionKind::Null)
                                }
                            }
                            ExpressionKind::FunctionNative(identifier) => {
                                // TO-DO check if the function arguments are correct or not.
                                let function = scope.get_declaration(identifier.clone()).unwrap();

                                if let Declaration::FunctionNative(function) = function {
                                    if let NativeArgument::Constant(function_list) = function.enter
                                    {
                                        let enter_a = function_list.len();
                                        let enter_b = list.len();

                                        if function_list.len() != list.len() {
                                            return Error::new_info(
                                                ErrorInfo::new_point(
                                                    self.span.clone(),
                                                    self.span.begin,
                                                    scope.get_active_source(),
                                                ),
                                                ErrorKind::InvalidInvocationArgumentLength(
                                                    identifier, enter_b, enter_a,
                                                ),
                                                None,
                                            );
                                        }

                                        for (i, target) in function_list.iter().enumerate() {
                                            let source = list[i].analyze(scope, infer.clone())?;

                                            if source != target.into_kind(scope) {
                                                panic!(
                                                    "native function: argument type mis-match ({source:?} != {target:?}) for function {:?}",
                                                    function.name,
                                                );
                                            }
                                        }
                                    }

                                    Ok(function.leave.into_kind(scope))
                                } else {
                                    panic!("invalid native function")
                                }
                            }
                            _ => panic!("invalid value for invocation operator {value:?}"),
                        }
                    }
                    ExpressionOperator::Indexation(expression) => {
                        // TO-DO check if expression is an integer type, return the index type (a is array, a[0] is integer)

                        match value {
                            ExpressionKind::Array(expression_kind) => Ok(*expression_kind),
                            ExpressionKind::Table(a, b) => Ok(*b),
                            _ => panic!("indexing a non-array value"),
                        }
                    }
                    _ => todo!(),
                }
            }
        }
    }

    pub fn compile_l(
        &self,
        scope: &Scope,
        function: &mut MFunction,
        from_dot: bool,
    ) -> Result<(), Error> {
        match &self.data {
            ExpressionData::Value(value) => match value {
                ExpressionValue::Identifier(identifier) => {
                    let value = scope
                        .get_declaration(identifier.clone())
                        .expect(&format!("no identifier {identifier}"));

                    match value {
                        Declaration::Definition(definition) => {
                            if from_dot {
                                function.push(Instruction::PushReference(definition.index.unwrap()))
                            } else {
                                function.push(Instruction::Save(definition.index.unwrap()))
                            }
                        }
                        _ => todo!(),
                    }
                }
                x => panic!("invalid L-expression value {x:?}"),
            },
            ExpressionData::Operation(operator, a, b) => match operator {
                ExpressionOperator::Dot => {
                    a.compile_l(scope, function, true)?;

                    let b = b.analyze_identifier()?;

                    if let ExpressionKind::Structure(identifier) = a.analyze(scope, None)?
                        && let Some(structure) = scope.get_structure(identifier)
                    {
                        function.push(Instruction::SaveField(
                            *structure.index_variable.get(&b.text).unwrap(),
                        ));
                    }

                    //function.push(Instruction::SaveField(b.text));

                    if !from_dot {
                        function.push(Instruction::SaveReference);
                    }
                }
                x => panic!("invalid L-expression operator {x:#?}"),
            },
            ExpressionData::OperationAfter(value, operator) => match operator {
                ExpressionOperator::Indexation(expression) => {
                    let kind = value.analyze(scope, None)?;

                    value.compile_l(scope, function, true)?;
                    expression.as_ref().unwrap().compile(scope, function)?;

                    match kind {
                        ExpressionKind::Array(_) => function.push(Instruction::SaveIndexArray),
                        ExpressionKind::Table(_, _) => function.push(Instruction::SaveIndexTable),
                        _ => todo!(),
                    }

                    if !from_dot {
                        function.push(Instruction::SaveReference);
                    }
                }
                _ => todo!(),
            },
            x => panic!("invalid L-expression type {x:#?}"),
        }

        Ok(())
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        match &self.data {
            ExpressionData::Value(value) => match value {
                ExpressionValue::Identifier(identifier) => {
                    let value = scope
                        .get_declaration(identifier.clone())
                        .expect(&format!("no declaration for identifier {identifier}"));

                    if let Declaration::Definition(definition) = value {
                        function.push(Instruction::Load(definition.index.unwrap()))
                    }
                }
                ExpressionValue::String(value) => {
                    function.push(Instruction::Push(Value::String(value.to_string())))
                }
                ExpressionValue::Integer(value) => {
                    function.push(Instruction::Push(Value::Integer(*value)))
                }
                ExpressionValue::Decimal(value) => {
                    function.push(Instruction::Push(Value::Decimal(*value)))
                }
                ExpressionValue::Boolean(value) => {
                    function.push(Instruction::Push(Value::Boolean(*value)))
                }
                ExpressionValue::Structure(value) => {
                    let structure = scope.get_structure(value.name.clone()).unwrap();

                    for field in structure.variable.keys().rev() {
                        let value = value.list.get(field).unwrap();
                        value.compile(scope, function)?;
                    }

                    function.push(Instruction::PushStructure(structure.index.unwrap()))
                }
                ExpressionValue::Enumerate(value) => {
                    let enumerate = scope.get_enumerate(value.name.clone()).unwrap();

                    for l in value.list.iter().rev() {
                        l.compile(scope, function)?;
                    }

                    function.push(Instruction::PushEnumerate(
                        enumerate.clone(),
                        value.kind.text.clone(),
                    ))
                }
                ExpressionValue::Array(value) => {
                    for l in value.list.iter().rev() {
                        l.compile(scope, function)?;
                    }

                    function.push(Instruction::PushArray(value.list.len()))
                }
                ExpressionValue::Table(value) => {
                    for (k, v) in value.list.iter().rev() {
                        k.compile(scope, function)?;
                        v.compile(scope, function)?;
                    }

                    function.push(Instruction::PushTable(value.list.len()))
                }
                _ => todo!(),
            },
            ExpressionData::Operation(operator, a, b) => {
                a.compile(scope, function)?;

                if let ExpressionOperator::Dot = operator {
                    let b = b.analyze_identifier()?;

                    if let ExpressionKind::Structure(identifier) = a.analyze(scope, None)?
                        && let Some(structure) = scope.get_structure(identifier)
                    {
                        function.push(Instruction::LoadField(
                            *structure.index_variable.get(&b.text).unwrap(),
                        ));
                    }

                    //function.push(Instruction::LoadField(b.text));

                    return Ok(());
                }

                b.compile(scope, function)?;

                match operator {
                    ExpressionOperator::Add => function.push(Instruction::Add),
                    ExpressionOperator::Subtract => function.push(Instruction::Subtract),
                    ExpressionOperator::Multiply => function.push(Instruction::Multiply),
                    ExpressionOperator::Divide => function.push(Instruction::Divide),
                    ExpressionOperator::And => function.push(Instruction::And),
                    ExpressionOperator::Or => function.push(Instruction::Or),
                    ExpressionOperator::GT => function.push(Instruction::GT),
                    ExpressionOperator::LT => function.push(Instruction::LT),
                    ExpressionOperator::Equal => function.push(Instruction::Equal),
                    ExpressionOperator::GTE => function.push(Instruction::GTE),
                    ExpressionOperator::LTE => function.push(Instruction::LTE),
                    ExpressionOperator::EqualNot => function.push(Instruction::EqualNot),
                    _ => todo!(),
                }
            }
            ExpressionData::OperationPrior(operator, value) => match operator {
                ExpressionOperator::Reference => {
                    let identifier = value.analyze_identifier()?;

                    let value = scope
                        .get_declaration(identifier.clone())
                        .expect(&format!("no declaration for identifier {identifier}"));

                    match value {
                        Declaration::Definition(definition) => {
                            function.push(Instruction::PushReference(definition.index.unwrap()));
                        }
                        _ => todo!(),
                    }
                }
                ExpressionOperator::Subtract => {
                    value.compile(scope, function)?;

                    function.push(Instruction::Negate)
                }
                ExpressionOperator::Not => {
                    value.compile(scope, function)?;

                    function.push(Instruction::Not)
                }
                _ => todo!(),
            },
            ExpressionData::OperationAfter(value, operator) => {
                let kind = value.analyze(scope, None)?;

                match operator {
                    ExpressionOperator::Invocation(list) => match kind {
                        ExpressionKind::Function(identifier) => {
                            let f = scope.get_function(identifier.clone()).unwrap();

                            println!("{} index: {:?}", identifier, f.index);

                            for argument in list.iter().rev() {
                                argument.compile(scope, function)?;
                            }

                            function.push(Instruction::Call(
                                f.index.unwrap(),
                                //FunctionCall::Function(identifier.text),
                                list.len(),
                            ))
                        }
                        ExpressionKind::FunctionNative(identifier) => {
                            for argument in list.iter().rev() {
                                argument.compile(scope, function)?;
                            }

                            function.push(Instruction::CallNative(identifier.text, list.len()))
                        }
                        _ => panic!("invalid value for invocation operator {value:?}"),
                    },
                    ExpressionOperator::Indexation(expression) => {
                        let kind = value.analyze(scope, None)?;

                        value.compile(scope, function)?;
                        expression.as_ref().unwrap().compile(scope, function)?;

                        match kind {
                            ExpressionKind::Array(_) => function.push(Instruction::LoadIndexArray),
                            ExpressionKind::Table(_, _) => {
                                function.push(Instruction::LoadIndexTable)
                            }
                            _ => todo!(),
                        }
                    }
                    _ => todo!(),
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub span: TokenSpan,
    pub name: Identifier,
    pub kind_i: Option<Kind>,
    pub kind_e: Option<ExpressionKind>,
    pub value: Expression,
    pub index: Option<usize>,
}

impl Definition {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Definition, |token_buffer| {
            token_buffer.want(TokenKind::Let)?;
            let name = token_buffer.want_identifier()?;

            let kind_i = {
                if token_buffer.want_peek(TokenKind::Colon) {
                    token_buffer.want(TokenKind::Colon)?;
                    Some(Kind::parse_token(token_buffer)?)
                } else {
                    None
                }
            };

            token_buffer.want(TokenKind::Definition)?;
            let value = Expression::parse_token(token_buffer, 0.0)?;
            token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self {
                span: token_buffer.get_span(),
                name,
                kind_i,
                kind_e: None,
                value,
                index: None,
            })
        })
    }

    pub fn analyze(&mut self, scope: &mut Scope) -> Result<ExpressionKind, Error> {
        let infer = if let Some(kind) = &self.kind_i {
            Some(kind.type_check(scope)?)
        } else {
            None
        };

        let source = self.value.analyze(scope, infer)?;

        if let Some(kind) = &self.kind_i {
            let target = kind.type_check(scope)?;

            if source != target {
                return Error::new_info(
                    ErrorInfo::new_point(self.span.clone(), None, scope.get_active_source()),
                    ErrorKind::IncorrectKind(target, source),
                    None,
                );
            }
        }

        self.kind_e = Some(source.clone());
        self.index = Some(scope.add_index_variable());

        scope.set_declaration(self.name.clone(), Declaration::Definition(self.clone()));

        Ok(source)
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        self.value.compile(scope, function)?;
        function.push(Instruction::Save(self.index.unwrap()));

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub span: TokenSpan,
    pub path: Expression,
    pub kind: Token,
    pub value: Expression,
}

impl Assignment {
    pub fn parse_token(token_buffer: &mut TokenBuffer, path: Expression) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Assignment, |token_buffer| {
            let kind = token_buffer.want_definition()?;
            let value = Expression::parse_token(token_buffer, 0.0)?;
            token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self {
                span: token_buffer.get_span(),
                path: path.clone(),
                kind,
                value,
            })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        self.value.analyze(scope, None)?;

        // TO-DO analyze if it's correct to load the value onto our path.

        Ok(())
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        self.value.compile(scope, function)?;
        self.path.compile_l(scope, function, false)?;

        Ok(())
    }
}

//================================================================

#[derive(Debug)]
struct Flow {
    //span: TokenSpan,
    list: Vec<Flow>,
    gate: bool,
    kind: Option<ExpressionKind>,
}

impl Flow {
    fn new(span: TokenSpan, gate: bool) -> Self {
        Self {
            //span,
            list: Vec::default(),
            gate,
            kind: None,
        }
    }

    fn kind(&self, all: bool) -> ExpressionKind {
        let mut kind = if let Some(kind) = &self.kind {
            kind.clone()
        } else {
            ExpressionKind::Null
        };
        let all_return = self.all_return();

        for flow in &self.list {
            let flow_kind = flow.kind(all || all_return);

            if kind != flow_kind {
                if kind == ExpressionKind::Null {
                    if all_return {
                        kind = flow_kind;
                    } else {
                        if !all {
                            panic!(
                                "type mis-match in flow return kind: {kind:?} != {:?}",
                                flow_kind
                            );
                        }
                    }
                } else {
                    if flow_kind != ExpressionKind::Null {
                        panic!(
                            "type mis-match in flow return kind: {kind:?} != {:?}",
                            flow_kind
                        );
                    }
                }
            }
        }

        kind
    }

    fn all_return(&self) -> bool {
        if self.list.is_empty() {
            return !self.gate && self.kind != Some(ExpressionKind::Null);
        }

        for flow in &self.list {
            // If this flow is reachable and does NOT guarantee a return → fail
            if !flow.gate && !flow.all_return() {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub span: TokenSpan,
    pub code: Vec<Statement>,
    pub scope: Option<ScopePointer>,
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
                    code.push(Statement::parse_token(token, token_buffer)?);
                }
            }

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self {
                span: token_buffer.get_span(),
                code,
                scope: None,
            })
        })
    }

    pub fn analyze(
        &mut self,
        scope: ScopePointer,
        argument: Vec<Variable>,
        iteration: bool,
    ) -> Result<Flow, Error> {
        let scope_block = Rc::new(RefCell::new(Scope::new(Some(scope))));

        for variable in &argument {
            let kind = variable.analyze(&scope_block.borrow())?;
            let index = scope_block.borrow_mut().add_index_variable();

            let definition = Definition {
                span: variable.span.clone(),
                name: variable.name.clone(),
                kind_i: Some(variable.kind.clone()),
                kind_e: Some(kind),
                // TO-DO will cause stack overflow
                value: Expression {
                    span: variable.span.clone(),
                    data: ExpressionData::Value(ExpressionValue::Identifier(variable.name.clone())),
                },
                index: Some(index),
            };

            scope_block
                .borrow_mut()
                .set_declaration(variable.name.clone(), Declaration::Definition(definition));
        }

        for statement in &self.code {
            match statement {
                Statement::Function(function) => scope_block.borrow_mut().set_declaration(
                    function.name.clone(),
                    Declaration::Function(function.clone()),
                ),
                Statement::Structure(structure) => scope_block.borrow_mut().set_declaration(
                    structure.name.clone(),
                    Declaration::Structure(structure.clone()),
                ),
                Statement::Enumerate(enumerate) => scope_block.borrow_mut().set_declaration(
                    enumerate.name.clone(),
                    Declaration::Enumerate(enumerate.clone()),
                ),
                _ => {}
            }
        }

        for statement in &mut self.code {
            match statement {
                Statement::Definition(definition) => {
                    definition.analyze(&mut scope_block.borrow_mut())?;
                }
                Statement::Assignment(assignment) => {
                    assignment.analyze(&scope_block.borrow())?;
                }
                Statement::Expression(expression) => {
                    expression.analyze(&scope_block.borrow(), None)?;
                }
                Statement::Condition(condition) => {
                    condition.analyze(scope_block.clone())?;
                }
                Statement::Iteration(iteration) => {
                    iteration.analyze(scope_block.clone())?;
                }
                Statement::Block(block) => {
                    block.analyze(scope_block.clone(), Vec::default(), false)?;
                }
                Statement::Skip => {
                    if !iteration {
                        // TO-DO use actual span data.
                        return Error::new_kind(ErrorKind::InvalidSkip, Some(ErrorHint::Iteration));
                    }
                }
                Statement::Exit => {
                    if !iteration {
                        // TO-DO use actual span data.
                        return Error::new_kind(ErrorKind::InvalidExit, Some(ErrorHint::Iteration));
                    }
                }
                Statement::Return(r) => {
                    r.analyze(&scope_block.borrow())?;
                }
                _ => {}
            }
        }

        let flow = self.analyze_flow(&scope_block.borrow(), false)?;

        self.scope = Some(scope_block);

        Ok(flow)
    }

    fn analyze_flow(&self, scope: &Scope, condition: bool) -> Result<Flow, Error> {
        let mut flow = Flow::new(self.span.clone(), condition);

        for statement in &self.code {
            match statement {
                Statement::Condition(condition) => {
                    flow.list.extend(condition.analyze_flow(scope)?);
                }
                Statement::Block(block) => {
                    flow.list.push(block.analyze_flow(scope, false)?);
                }
                Statement::Return(r) => {
                    flow.kind = Some(r.analyze(scope)?);
                }
                _ => {}
            }
        }

        Ok(flow)
    }

    #[rustfmt::skip]
    pub fn compile(&self, scope: &Scope, function: &mut MFunction, root: bool, header: Option<usize>) -> Result<(), Error> {
        let block = self.scope.as_ref().unwrap();
        let mut variable_a = scope.get_index_variable();
        let mut variable_b = scope.get_index_variable();
        let mut exit = Vec::new();

        for statement in &self.code {
            match statement {
                Statement::Definition(definition) => {
                    definition.compile(&block.borrow(), function)?;

                    if !root {
                        variable_b += 1;
                    }
                },
                Statement::Assignment(assignment) => { assignment.compile(&block.borrow(), function)?; },
                Statement::Expression(expression) => { expression.compile(&block.borrow(), function)?; },
                Statement::Condition(condition)   => { condition.compile(&block.borrow(), function)?;  },
                Statement::Iteration(iteration)   => { iteration.compile(&block.borrow(), function)?;  },
                Statement::Block(b)               => { b.compile(&block.borrow(), function, false, None)?; },
                Statement::Skip => if let Some(header) = header {
                    for v in variable_a..variable_b {
                        function.push(Instruction::Hide(v));
                    }

                    function.push(Instruction::Jump(header));
                },
                Statement::Exit => {
                    for v in variable_a..variable_b {
                        function.push(Instruction::Hide(v));
                    }

                    exit.push(function.cursor());

                    function.push(Instruction::Null);
                },
                Statement::Return(r) => r.compile(&block.borrow(), function)?,
                _ => {}
            }
        }

        if !root {
            for v in variable_a..variable_b {
                function.push(Instruction::Hide(v));
            }
        }

        for e in exit {
            function.change(Instruction::Jump(function.cursor() + 1), e);
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub span: TokenSpan,
    pub name: Identifier,
    pub enter: Vec<Variable>,
    pub leave: Option<Kind>,
    pub block: Block,
    pub method: bool,
    pub index: Option<usize>,
}

impl Function {
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
        parent: Option<Identifier>,
    ) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Function, |token_buffer| {
            token_buffer.want(TokenKind::Function)?;

            let name = token_buffer.want_identifier()?;
            let mut enter = Vec::new();
            let mut leave = None;
            let mut method = false;

            token_buffer.want(TokenKind::ParenthesisBegin)?;

            // No argument branch.
            if token_buffer.want_peek(TokenKind::ParenthesisClose) {
                token_buffer.want(TokenKind::ParenthesisClose)?;
            } else {
                let mut first = true;

                Statement::parse_comma(
                    token_buffer,
                    TokenKind::ParenthesisClose,
                    |token_buffer| {
                        if first {
                            if token_buffer.want_peek(TokenKind::SelfLower) {
                                method = true;
                            }
                        }

                        enter.push(Variable::parse_token(token_buffer, parent.clone())?);

                        first = false;

                        Ok(())
                    },
                )?;

                token_buffer.want(TokenKind::ParenthesisClose)?;
            }

            if token_buffer.want_peek(TokenKind::Colon) {
                token_buffer.want(TokenKind::Colon)?;

                if token_buffer.want_peek(TokenKind::SelfUpper) {
                    token_buffer.want(TokenKind::SelfUpper)?;

                    if let Some(parent) = &parent {
                        leave = Some(Kind {
                            name: parent.clone(),
                            list: Vec::default(),
                            reference: false,
                        });
                    } else {
                        panic!("self in non-structure/enumerate")
                    }
                } else {
                    leave = Some(Kind::parse_token(token_buffer)?);
                }
            }

            let block = Block::parse_token(token_buffer)?;

            Ok(Self {
                span: token_buffer.get_span(),
                name,
                enter,
                leave,
                block,
                method,
                index: None,
            })
        })
    }

    pub fn analyze(&mut self, scope: ScopePointer) -> Result<(), Error> {
        let flow = self
            .block
            .analyze(scope.clone(), self.enter.clone(), false)?;

        let target = if let Some(leave) = &self.leave {
            leave.type_check(&scope.borrow())?
        } else {
            ExpressionKind::Null
        };
        let source = flow.kind(false);

        if source != target {
            return Error::new_info(
                ErrorInfo::new_point(self.span.clone(), None, scope.borrow().get_active_source()),
                ErrorKind::IncorrectKind(target, source),
                None,
            );
        }

        self.index = Some(scope.borrow_mut().add_index_function());

        Ok(())
    }

    pub fn compile(&self, scope: &Scope) -> Result<MFunction, Error> {
        let mut function = MFunction::default();

        for parameter in &self.enter {
            function.push_parameter(parameter.name.text.clone());
        }

        self.block.compile(scope, &mut function, true, None)?;

        Ok(function)
    }
}

#[derive(Debug, Clone)]
pub struct Kind {
    pub name: Identifier,
    pub list: Vec<Self>,
    pub reference: bool,
}

impl Kind {
    fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Kind, |token_buffer| {
            let reference = if token_buffer.want_peek(TokenKind::Ampersand) {
                token_buffer.want(TokenKind::Ampersand)?;
                true
            } else {
                false
            };

            let name = token_buffer.want_identifier()?;
            let mut list = Vec::new();

            if token_buffer.want_peek(TokenKind::LT) {
                token_buffer.want(TokenKind::LT)?;

                Statement::parse_comma(token_buffer, TokenKind::GT, |token_buffer| {
                    list.push(Kind::parse_token(token_buffer)?);
                    Ok(())
                })?;

                token_buffer.want(TokenKind::GT)?;
            }

            Ok(Self {
                name,
                list,
                reference,
            })
        })
    }

    fn type_check(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
        Ok(match self.name.text.as_str() {
            "String" => ExpressionKind::String,
            "Integer" => ExpressionKind::Integer,
            "Decimal" => ExpressionKind::Decimal,
            "Boolean" => ExpressionKind::Boolean,
            "Array" => {
                let first = self.list.get(0).unwrap();
                ExpressionKind::Array(Box::new(first.type_check(scope)?))
            }
            "Table" => {
                let k = self.list.get(0).unwrap();
                let v = self.list.get(1).unwrap();
                ExpressionKind::Table(
                    Box::new(k.type_check(scope)?),
                    Box::new(v.type_check(scope)?),
                )
            }
            _ => {
                let definition = scope
                    .get_declaration(self.name.clone())
                    .expect(&format!("no declaration for name {:?}", self.name));

                match definition {
                    Declaration::Structure(structure) => {
                        ExpressionKind::Structure(structure.name.clone())
                    }
                    Declaration::Enumerate(enumerate) => {
                        ExpressionKind::Enumerate(enumerate.name.clone())
                    }
                    _ => panic!("type_check: definition is not a structure or enumeration"),
                }
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub span: TokenSpan,
    pub name: Identifier,
    pub kind: Kind,
}

impl Variable {
    fn parse_token(
        token_buffer: &mut TokenBuffer,
        parent: Option<Identifier>,
    ) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Variable, |token_buffer| {
            let (name, kind) = if token_buffer.want_peek(TokenKind::SelfLower) {
                token_buffer.want(TokenKind::SelfLower)?;

                if let Some(parent) = &parent {
                    (
                        // TO-DO use self-lower span?
                        Identifier::from_string("self".to_string(), Point::default()).unwrap(),
                        Kind {
                            name: parent.clone(),
                            list: Vec::default(),
                            reference: false,
                        },
                    )
                } else {
                    panic!("self on non-structure/enumerate")
                }
            } else {
                let name = token_buffer.want_identifier()?;
                token_buffer.want(TokenKind::Colon)?;

                (name, Kind::parse_token(token_buffer)?)
            };

            Ok(Self {
                span: token_buffer.get_span(),
                name,
                kind,
            })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
        self.kind.type_check(scope)
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct ArrayD {
    pub list: Vec<Expression>,
}

impl ArrayD {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Array, |token_buffer| {
            let mut list = Vec::new();

            token_buffer.want(TokenKind::SquareBegin)?;

            Statement::parse_comma(token_buffer, TokenKind::SquareClose, |token_buffer| {
                list.push(Expression::parse_token(token_buffer, 0.0)?);
                Ok(())
            })?;

            token_buffer.want(TokenKind::SquareClose)?;

            Ok(Self { list })
        })
    }

    pub fn analyze(
        &self,
        scope: &Scope,
        infer: Option<ExpressionKind>,
    ) -> Result<ExpressionKind, Error> {
        let infer = if let Some(infer) = infer {
            match infer {
                ExpressionKind::Array(kind) => Some(*kind),
                x => panic!("non-array kind for array definition {x:?}"),
            }
        } else {
            None
        };

        let mut current = infer;

        for expression in &self.list {
            let kind = expression.analyze(scope, current.clone())?;

            if let Some(ref current) = current {
                if kind != *current {
                    panic!("type mis-match in array literal ({kind:?} != {current:?})")
                }
            } else {
                current = Some(kind)
            }
        }

        Ok(ExpressionKind::Array(Box::new(
            current.expect("could not infer type for array"),
        )))
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct TableD {
    pub list: Vec<(Expression, Expression)>,
}

impl TableD {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        // TO-DO use Hint::Table
        token_buffer.parse(ErrorHint::Array, |token_buffer| {
            let mut list = Vec::new();

            token_buffer.want(TokenKind::CurlyBegin)?;

            Statement::parse_comma(token_buffer, TokenKind::CurlyClose, |token_buffer| {
                let k = Expression::parse_token(token_buffer, 0.0)?;
                token_buffer.want(TokenKind::Definition)?;
                let v = Expression::parse_token(token_buffer, 0.0)?;

                list.push((k, v));

                Ok(())
            })?;

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self { list })
        })
    }

    pub fn analyze(
        &self,
        scope: &Scope,
        infer: Option<ExpressionKind>,
    ) -> Result<ExpressionKind, Error> {
        let (i_a, i_b) = if let Some(infer) = infer {
            match infer {
                ExpressionKind::Table(a, b) => (Some(*a), Some(*b)),
                x => panic!("non-table kind for table definition {x:?}"),
            }
        } else {
            (None, None)
        };

        let mut c_a = i_a;
        let mut c_b = i_b;

        for (e_a, e_b) in &self.list {
            let k_a = e_a.analyze(scope, c_a.clone())?;
            let k_b = e_b.analyze(scope, c_b.clone())?;

            if let Some(ref c_a) = c_a {
                if k_a != *c_a {
                    panic!("type mis-match in array literal ({k_a:?} != {c_a:?})")
                }
            } else {
                c_a = Some(k_a)
            }

            if let Some(ref c_b) = c_b {
                if k_b != *c_b {
                    panic!("type mis-match in array literal ({k_b:?} != {c_b:?})")
                }
            } else {
                c_b = Some(k_b)
            }
        }

        Ok(ExpressionKind::Table(
            Box::new(c_a.unwrap()),
            Box::new(c_b.unwrap()),
        ))
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct StructureD {
    pub name: Identifier,
    pub list: BTreeMap<String, Expression>,
}

impl StructureD {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::StructureD, |token_buffer| {
            let mut list = BTreeMap::new();

            let name = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::CurlyBegin)?;

            Statement::parse_comma(token_buffer, TokenKind::CurlyClose, |token_buffer| {
                let name = token_buffer.want_identifier()?.text;
                token_buffer.want(TokenKind::Definition)?;
                let value = Expression::parse_token(token_buffer, 0.0)?;

                list.insert(name, value);
                Ok(())
            })?;

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self { name, list })
        })
    }

    fn analyze(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
        let structure = scope.get_structure(self.name.clone()).unwrap();

        if self.list.len() != structure.variable.len() {
            panic!("structure literal: mis-match in field count")
        }

        for (field, variable) in &structure.variable {
            let value = self.list.get(field).unwrap();
            let target = variable.analyze(scope)?;
            let source = value.analyze(scope, Some(target.clone()))?;

            if source != target {
                panic!(
                    "structure literal: type mis-match ({source:?} != {target:?}) for field {field}"
                )
            }
        }

        Ok(ExpressionKind::Structure(self.name.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub name: Identifier,
    pub kind: Option<Vec<Identifier>>,
    pub parent: Option<Identifier>,
    pub variable: BTreeMap<String, Variable>,
    pub function: BTreeMap<String, Function>,
    pub index: Option<usize>,
    pub index_variable: HashMap<String, usize>,
}

impl Structure {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Structure, |token_buffer| {
            let mut variable = BTreeMap::new();
            let mut function = BTreeMap::new();
            let mut index_variable = HashMap::default();

            token_buffer.want(TokenKind::Structure)?;

            let name = token_buffer.want_identifier()?;

            let kind = if token_buffer.want_peek(TokenKind::LT) {
                let mut kind = Vec::new();

                token_buffer.want(TokenKind::LT)?;

                Statement::parse_comma(token_buffer, TokenKind::GT, |token_buffer| {
                    kind.push(token_buffer.want_identifier()?);
                    Ok(())
                })?;

                token_buffer.want(TokenKind::GT)?;

                Some(kind)
            } else {
                None
            };

            let parent = if token_buffer.want_peek(TokenKind::Colon) {
                token_buffer.want(TokenKind::Colon)?;

                Some(token_buffer.want_identifier()?)
            } else {
                None
            };

            token_buffer.want(TokenKind::CurlyBegin)?;

            while let Some(token) = token_buffer.peek() {
                if token.class.kind() == TokenKind::CurlyClose {
                    break;
                }

                if token.class.kind() == TokenKind::Function {
                    let f = Function::parse_token(token_buffer, Some(name.clone()))?;
                    function.insert(f.name.text.clone(), f);
                } else if token.class.kind() == TokenKind::Identifier {
                    let v = Variable::parse_token(token_buffer, None)?;
                    index_variable.insert(v.name.text.clone(), variable.len());
                    variable.insert(v.name.text.clone(), v);

                    if let Some(token) = token_buffer.peek()
                        && token.class.kind() == TokenKind::Comma
                    {
                        token_buffer.next();
                    } else {
                        break;
                    }
                }
            }

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self {
                name,
                kind,
                parent,
                variable,
                function,
                index: None,
                index_variable,
            })
        })
    }

    pub fn analyze(&mut self, scope: ScopePointer) -> Result<ExpressionKind, Error> {
        for variable in self.variable.values() {
            variable.analyze(&scope.borrow())?;
        }

        for function in self.function.values_mut() {
            function.analyze(scope.clone())?;
        }

        self.index = Some(scope.borrow_mut().add_index_structure());

        Ok(ExpressionKind::Structure(self.name.clone()))
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct EnumerateD {
    pub name: Identifier,
    pub kind: Identifier,
    pub list: Vec<Expression>,
}

impl EnumerateD {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::EnumerateD, |token_buffer| {
            let mut list = Vec::new();

            let name = token_buffer.want_identifier()?;
            token_buffer.want(TokenKind::Colon)?;
            let kind = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::CurlyBegin)?;

            Statement::parse_comma(token_buffer, TokenKind::CurlyClose, |token_buffer| {
                list.push(Expression::parse_token(token_buffer, 0.0)?);
                Ok(())
            })?;

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self { name, kind, list })
        })
    }
}

#[derive(Debug, Clone)]
pub struct Enumerate {
    pub name: Identifier,
    pub variable: BTreeMap<String, Vec<Identifier>>,
    pub function: BTreeMap<String, Function>,
    pub index: Option<usize>,
}

impl Enumerate {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Enumerate, |token_buffer| {
            let mut variable = BTreeMap::new();
            let mut function = BTreeMap::new();

            token_buffer.want(TokenKind::Enumerate)?;

            let name = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::CurlyBegin)?;

            while let Some(token) = token_buffer.peek() {
                if token.class.kind() == TokenKind::CurlyClose {
                    break;
                }

                if token.class.kind() == TokenKind::Function {
                    let f = Function::parse_token(token_buffer, Some(name.clone()))?;
                    function.insert(f.name.text.clone(), f);
                } else if token.class.kind() == TokenKind::Identifier {
                    let name = token_buffer.want_identifier()?;
                    let mut kind = Vec::new();

                    if token_buffer.want_peek(TokenKind::ParenthesisBegin) {
                        token_buffer.want(TokenKind::ParenthesisBegin)?;

                        Statement::parse_comma(
                            token_buffer,
                            TokenKind::ParenthesisClose,
                            |token_buffer| {
                                kind.push(token_buffer.want_identifier()?);
                                Ok(())
                            },
                        )?;

                        token_buffer.want(TokenKind::ParenthesisClose)?;
                    }

                    variable.insert(name.text, kind);

                    if let Some(token) = token_buffer.peek()
                        && token.class.kind() == TokenKind::Comma
                    {
                        token_buffer.next();
                    } else {
                        break;
                    }
                }
            }

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self {
                name,
                variable,
                function,
                index: None,
            })
        })
    }

    pub fn analyze(&mut self, scope: ScopePointer) -> Result<(), Error> {
        for function in self.function.values_mut() {
            function.analyze(scope.clone())?;
        }

        self.index = Some(scope.borrow_mut().add_index_enumerate());

        Ok(())
    }
}
