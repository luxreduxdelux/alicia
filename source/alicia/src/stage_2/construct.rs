use crate::helper::error::*;
use crate::stage_1::buffer::*;
use crate::stage_1::helper::*;
use crate::stage_1::token::*;
use crate::stage_2::scope::*;
use crate::stage_4::machine::Function as MFunction;
use crate::stage_4::machine::FunctionCall;
use crate::stage_4::machine::Instruction;
use crate::stage_4::machine::Value;

//================================================================

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Display;

//================================================================

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
        // TO-DO does not account for +=, -=, etc.
        if let Some(token) = token_buffer.peek_ahead(1)
            && token.class.kind() == TokenKind::Definition
        {
            return Ok(Self::Assignment(Assignment::parse_token(token_buffer)?));
        }

        let e = Expression::parse_token(token_buffer, 0.0)?;

        token_buffer.want(TokenKind::ColonSemi)?;

        Ok(Self::Expression(e))
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

struct ConditionBody {
    head: usize,
    code: usize,
    tail: usize,
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

    pub fn analyze(&mut self, scope: &mut Scope) -> Result<(), Error> {
        if let Some(value) = &self.value {
            let kind = value.analyze(scope)?;

            if kind != ExpressionKind::Boolean {
                panic!("condition expression kind is not a boolean");
            }
        }

        if let Some(child) = &mut self.child {
            child.analyze(scope)?;
        }

        self.block.analyze(scope, false)?;

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
            value.analyze(scope)
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

    pub fn analyze(&mut self, scope: &mut Scope) -> Result<(), Error> {
        if let Some(value) = &self.value {
            match value {
                IterationValue::Iterational(assignment) => assignment.analyze(scope)?,
                IterationValue::Conditional(expression) => {
                    expression.analyze(scope)?;
                }
            };
        }

        self.block.analyze(scope, true)?;

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
            function.change(Instruction::Branch(function.cursor()), branch);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Eq)]
pub enum ExpressionKind {
    Null,
    Path,
    String,
    Integer,
    Decimal,
    Boolean,
    Structure(Identifier),
    Enumerate(Identifier),
    Array,
}

impl PartialEq for ExpressionKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Structure(l0), Self::Structure(r0)) => l0.text == r0.text,
            (Self::Enumerate(l0), Self::Enumerate(r0)) => l0.text == r0.text,
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
    Path(Path),
    String(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    Structure(StructureD),
    Array(ArrayD),
}

impl Display for ExpressionValue {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null             => formatter.write_str("Null"),
            Self::Path(_)          => formatter.write_str("Path"),
            Self::String(value)    => formatter.write_str(&value.to_string()),
            Self::Integer(value)   => formatter.write_str(&value.to_string()),
            Self::Decimal(value)   => formatter.write_str(&value.to_string()),
            Self::Boolean(value)   => formatter.write_str(&value.to_string()),
            _ => todo!(),
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
                TokenClass::SquareBegin => Ok(Self::Array(ArrayD::parse_token(token_buffer)?)),
                _ => panic!(
                    "Alicia internal error: ExpressionValue::parse_token(): want_token() gave back a token that is not a possible value"
                ),
            }
        } else {
            panic!("TO-DO from_token")
        }
    }

    #[rustfmt::skip]
    pub fn kind(&self) -> ExpressionKind {
        match self {
            Self::Null          => ExpressionKind::Null,
            Self::Path(_)       => ExpressionKind::Path,
            Self::String(_)     => ExpressionKind::String,
            Self::Integer(_)    => ExpressionKind::Integer,
            Self::Decimal(_)    => ExpressionKind::Decimal,
            Self::Boolean(_)    => ExpressionKind::Boolean,
            Self::Structure(x)  => ExpressionKind::Structure(x.name.clone()),
            //Self::Enumerate(x)  => ExpressionKind::Enumerate(x.name.clone()),
            Self::Array(_)      => ExpressionKind::Array,
        }
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
    Reference,
}

impl ExpressionOperator {
    #[rustfmt::skip]
    fn from_token(token: Token) -> Self {
        match token.class.kind() {
            TokenKind::Add       => Self::Add,
            TokenKind::Subtract  => Self::Subtract,
            TokenKind::Multiply  => Self::Multiply,
            TokenKind::Divide    => Self::Divide,
            TokenKind::Not       => Self::Not,
            TokenKind::And       => Self::And,
            TokenKind::Or        => Self::Or,
            TokenKind::GT        => Self::GT,
            TokenKind::LT        => Self::LT,
            TokenKind::Equal     => Self::Equal,
            TokenKind::GTE       => Self::GTE,
            TokenKind::LTE       => Self::LTE,
            TokenKind::EqualNot  => Self::EqualNot,
            TokenKind::Ampersand => Self::Reference,
            _ => panic!(
                "Alicia internal error: ExpressionValue::parse_token(): want_token() gave back a token that is not a possible value"
            ),
        }
    }

    #[rustfmt::skip]
    fn parse_token_mono(&self, token_a: Expression) -> Expression {
        let token_a = Box::new(token_a);

        match self {
            Self::Subtract  => Expression::OperationPrior(Self::Subtract, token_a),
            Self::Reference => Expression::OperationPrior(Self::Reference, token_a),
            x => panic!("incorrect parse_token_mono operator: {x:?}")
        }
    }

    #[rustfmt::skip]
    fn parse_token_binary(&self, token_a: Expression, token_b: Expression) -> Expression {
        let token_a = Box::new(token_a);
        let token_b = Box::new(token_b);

        match self {
            Self::Add      => Expression::Operation(Self::Add,      token_a, token_b),
            Self::Subtract => Expression::Operation(Self::Subtract, token_a, token_b),
            Self::Multiply => Expression::Operation(Self::Multiply, token_a, token_b),
            Self::Divide   => Expression::Operation(Self::Divide,   token_a, token_b),
            Self::And      => Expression::Operation(Self::And,      token_a, token_b),
            Self::Or       => Expression::Operation(Self::Or,       token_a, token_b),
            Self::GT       => Expression::Operation(Self::GT,       token_a, token_b),
            Self::LT       => Expression::Operation(Self::LT,       token_a, token_b),
            Self::Equal    => Expression::Operation(Self::Equal,    token_a, token_b),
            Self::GTE      => Expression::Operation(Self::GTE,      token_a, token_b),
            Self::LTE      => Expression::Operation(Self::LTE,      token_a, token_b),
            Self::EqualNot => Expression::Operation(Self::EqualNot, token_a, token_b),
            x => panic!("incorrect parse_token_binary operator: {x:?}")
        }
    }

    #[rustfmt::skip]
    fn bind_power(&self) -> (f32, f32) {
        match self {
            Self::Add       => (1.0, 1.1),
            Self::Subtract  => (1.0, 1.1),
            Self::Multiply  => (2.0, 2.1),
            Self::Divide    => (2.0, 2.1),
            // TO-DO add actual bind power to these
            Self::Not       => (1.0, 1.1),
            Self::And       => (1.0, 1.1),
            Self::Or        => (1.0, 1.1),
            Self::GT        => (1.0, 1.1),
            Self::LT        => (1.0, 1.1),
            Self::Equal     => (1.0, 1.1),
            Self::GTE       => (1.0, 1.1),
            Self::LTE       => (1.0, 1.1),
            Self::EqualNot  => (1.0, 1.1),
            Self::Reference => (2.1, 2.0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Value(ExpressionValue),
    Operation(ExpressionOperator, Box<Expression>, Box<Expression>),
    OperationPrior(ExpressionOperator, Box<Expression>),
}

impl Expression {
    pub fn parse_token(token_buffer: &mut TokenBuffer, bind_power: f32) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Expression, |token_buffer| {
            let mut value_a = if token_buffer.want_peek(TokenKind::ParenthesisBegin) {
                token_buffer.want(TokenKind::ParenthesisBegin)?;
                let value = Self::parse_token(token_buffer, 0.0)?;
                token_buffer.want(TokenKind::ParenthesisClose)?;

                value
            } else if let Some(token) = token_buffer.peek_operator() {
                let operator = ExpressionOperator::from_token(token);

                token_buffer.want_operator()?;

                ExpressionOperator::parse_token_mono(
                    &operator,
                    Expression::Value(ExpressionValue::from_token(token_buffer)?),
                )
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

                value_a = ExpressionOperator::parse_token_binary(&operator, value_a, value_b)
            }

            Ok(value_a)
        })
    }

    #[rustfmt::skip]
    pub fn analyze(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
        match self {
            Expression::Value(value) => match value {
                ExpressionValue::Path(path) => path.analyze(scope),
                _ => Ok(value.kind()),
            },
            Expression::Operation(operator, e_a, e_b) => {
                let a = e_a.analyze(scope)?;
                let b = e_b.analyze(scope)?;

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
                        ExpressionOperator::Add      => Ok(a),
                        ExpressionOperator::Subtract => Ok(a),
                        ExpressionOperator::Multiply => Ok(a),
                        ExpressionOperator::Divide   => Ok(a),
                        ExpressionOperator::GT       => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::LT       => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::Equal    => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::GTE      => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::LTE      => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::EqualNot => Ok(ExpressionKind::Boolean),
                        _ => panic!("unsupported operator {operator:?} for value of type {a:?}"),
                    }
                } else if a == ExpressionKind::Boolean {
                    match operator {
                        ExpressionOperator::And      => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::Or       => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::Equal    => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::EqualNot => Ok(ExpressionKind::Boolean),
                        _ => panic!("unsupported operator {operator:?} for value of type {a:?}"),
                    }
                } else {
                    match operator {
                        ExpressionOperator::Equal    => Ok(ExpressionKind::Boolean),
                        ExpressionOperator::EqualNot => Ok(ExpressionKind::Boolean),
                        _ => panic!("unsupported operator {operator:?} for value of type {a:?}"),
                    }
                }
            }
            _ => todo!(),
        }
    }

    #[rustfmt::skip]
    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        match self {
            Expression::Value(value) => match value {
                ExpressionValue::Path(path) => path.compile(scope, function)?,
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
                    let structure = scope.get_declaration(value.name.clone()).unwrap();

                    match structure {
                        Declaration::Structure(structure) => {

                            for field in structure.variable.keys().rev() {
                                let assignment = value.list.get(field).unwrap();
                                assignment.value.compile(scope, function)?;
                            }

                            function.push(Instruction::PushStructure(structure.clone()))
                        },
                        x => panic!("declaration is not a structure: {x:?}")
                    }
                }
                _ => todo!(),
            },
            Expression::Operation(operator, a, b) => {
                a.compile(scope, function)?;
                b.compile(scope, function)?;

                match operator {
                    ExpressionOperator::Add      => function.push(Instruction::Add),
                    ExpressionOperator::Subtract => function.push(Instruction::Subtract),
                    ExpressionOperator::Multiply => function.push(Instruction::Multiply),
                    ExpressionOperator::Divide   => function.push(Instruction::Divide),
                    ExpressionOperator::And      => function.push(Instruction::And),
                    ExpressionOperator::Or       => function.push(Instruction::Or),
                    ExpressionOperator::GT       => function.push(Instruction::GT),
                    ExpressionOperator::LT       => function.push(Instruction::LT),
                    ExpressionOperator::Equal    => function.push(Instruction::Equal),
                    ExpressionOperator::GTE      => function.push(Instruction::GTE),
                    ExpressionOperator::LTE      => function.push(Instruction::LTE),
                    ExpressionOperator::EqualNot => function.push(Instruction::EqualNot),
                    // TO-DO reference
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub span: TokenSpan,
    pub name: Identifier,
    pub kind_i: Option<Identifier>,
    pub kind_e: Option<ExpressionKind>,
    pub value: Expression,
}

impl Definition {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Definition, |token_buffer| {
            token_buffer.want(TokenKind::Let)?;
            let name = token_buffer.want_identifier()?;

            let kind_i = {
                if token_buffer.want_peek(TokenKind::Colon) {
                    token_buffer.want(TokenKind::Colon)?;
                    Some(token_buffer.want_identifier()?)
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
            })
        })
    }

    pub fn analyze(&mut self, scope: &mut Scope) -> Result<ExpressionKind, Error> {
        let source = self.value.analyze(scope)?;

        if let Some(kind) = &self.kind_i {
            let target = Variable::type_check(&self.span, kind, scope)?;

            if source != target {
                return Err(Error::new_info(
                    ErrorInfo::new_point(self.span.clone(), None),
                    ErrorKind::IncorrectKind(target, source),
                    None,
                ));
            }
        }

        self.kind_e = Some(source.clone());

        scope.set_declaration(self.name.clone(), Declaration::Definition(self.clone()));

        Ok(source)
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        self.value.compile(scope, function)?;
        function.push(Instruction::Save(self.name.text.clone()));

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub span: TokenSpan,
    pub path: Path,
    pub kind: Token,
    pub value: Expression,
}

impl Assignment {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Assignment, |token_buffer| {
            let path = Path::parse_token(token_buffer)?;
            let kind = token_buffer.want_definition()?;
            let value = Expression::parse_token(token_buffer, 0.0)?;
            token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self {
                span: token_buffer.get_span(),
                path,
                kind,
                value,
            })
        })
    }

    pub fn parse_token_loose(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Assignment, |token_buffer| {
            let path = Path::parse_token(token_buffer)?;
            let kind = token_buffer.want_definition()?;
            let value = Expression::parse_token(token_buffer, 0.0)?;

            Ok(Self {
                span: token_buffer.get_span(),
                path,
                kind,
                value,
            })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        /*
        if let Some(variable) = scope.get_declaration(self.path.clone()) {
            match variable {
                Declaration::Definition(_) => {
                    Ok(())
                    // TO-DO type check that whatever we're assigning to the
                    // definition is valid.
                }
                _ => Err(Error::new_info(
                    ErrorInfo::new_point(self.span.clone(), Some(self.path.point)),
                    ErrorKind::InvalidAssignment(self.path.clone()),
                    Some(ErrorHint::Assignment),
                )),
            }
        } else {
            Err(Error::new_info(
                ErrorInfo::new_point(self.span.clone(), Some(self.path.point)),
                ErrorKind::UnknownSymbol(self.path.clone()),
                Some(ErrorHint::Assignment),
            ))
        }?;
        */

        self.value.analyze(scope)?;

        Ok(())
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        //self.value.compile(scope, function)?;
        //function.push(Instruction::Save(self.name.text.clone()));

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

            //token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self {
                span: token_buffer.get_span(),
                name,
                list,
            })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
        if let Some(declaration) = scope.get_declaration(self.name.clone()) {
            match declaration {
                Declaration::Function(function) => {
                    if self.list.len() != function.enter.len() {
                        panic!("analyze: incorrect argument count for function");
                    }

                    for (i, expression) in self.list.iter().enumerate() {
                        let source = expression.analyze(scope)?;
                        let target = &function.enter[i];
                        let target = Variable::type_check(&target.span, &target.kind, scope)?;

                        if source != target {
                            return Err(Error::new_info(
                                ErrorInfo::new_point(self.span.clone(), None),
                                ErrorKind::IncorrectKind(target, source),
                                None,
                            ));
                        }
                    }

                    if let Some(leave) = &function.leave {
                        return Variable::type_check(&function.span, leave, scope);
                    }

                    Ok(ExpressionKind::Null)
                }
                Declaration::FunctionNative(function) => {
                    //if self.list.len() != function.enter.len() {
                    //    panic!("analyze: incorrect argument count for function native");
                    //}

                    // TO-DO type check and validate the function is receiving every argument
                    for expression in &self.list {
                        expression.analyze(scope)?;
                    }

                    Ok(function.leave.clone())
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

    pub fn compile(&self, _: &Scope) -> Result<(), Error> {
        Ok(())
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

            //token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self { name, expression })
        })
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
    pub scope: Option<Scope>,
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

    pub fn analyze(&mut self, scope: &mut Scope, iteration: bool) -> Result<(), Error> {
        let mut scope_block = Scope::new(Some(Box::new(scope)));

        for statement in &self.code {
            match statement {
                Statement::Function(function) => scope_block.set_declaration(
                    function.name.clone(),
                    Declaration::Function(function.clone()),
                ),
                Statement::Structure(structure) => scope_block.set_declaration(
                    structure.name.clone(),
                    Declaration::Structure(structure.clone()),
                ),
                Statement::Enumerate(enumerate) => scope_block.set_declaration(
                    enumerate.name.clone(),
                    Declaration::Enumerate(enumerate.clone()),
                ),
                _ => {}
            }
        }

        for statement in &mut self.code {
            match statement {
                Statement::Definition(definition) => {
                    definition.analyze(&mut scope_block)?;
                }
                Statement::Assignment(assignment) => {
                    assignment.analyze(&scope_block)?;
                }
                Statement::Expression(expression) => {
                    expression.analyze(&scope_block)?;
                }
                Statement::Condition(condition) => {
                    condition.analyze(&mut scope_block)?;
                }
                Statement::Iteration(iteration) => {
                    iteration.analyze(&mut scope_block)?;
                }
                Statement::Block(block) => {
                    block.analyze(&mut scope_block, false)?;
                }
                Statement::Skip => {
                    if !iteration {
                        // TO-DO use actual span data.
                        return Err(Error::new_kind(
                            ErrorKind::InvalidSkip,
                            Some(ErrorHint::Iteration),
                        ));
                    }
                }
                Statement::Exit => {
                    if !iteration {
                        // TO-DO use actual span data.
                        return Err(Error::new_kind(
                            ErrorKind::InvalidExit,
                            Some(ErrorHint::Iteration),
                        ));
                    }
                }
                Statement::Return(r) => {
                    r.analyze(&scope_block)?;
                }
                _ => {}
            }
        }

        self.scope = Some(scope_block);

        Ok(())
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
    pub fn compile(&self, _: &Scope, function: &mut MFunction, root: bool, header: Option<usize>) -> Result<(), Error> {
        let block = self.scope.as_ref().unwrap();
        let mut variable = Vec::new();
        let mut exit = Vec::new();

        for statement in &self.code {
            match statement {
                Statement::Definition(definition) => {
                    definition.compile(block, function)?;

                    if !root {
                        variable.push(definition.name.text.clone());
                    }
                },
                Statement::Assignment(assignment) => { assignment.compile(block, function)?; },
                Statement::Expression(expression) => { expression.compile(block, function)?; },
                Statement::Condition(condition)   => { condition.compile(block, function)?;  },
                Statement::Iteration(iteration)   => { iteration.compile(block, function)?;  },
                Statement::Block(b)               => b.compile(block, function, false, None)?,
                Statement::Skip                   => if let Some(header) = header {
                    for v in &variable {
                        function.push(Instruction::Hide(v.to_string()));
                    }

                    function.push(Instruction::Jump(header));
                },
                Statement::Exit                   => {
                    for v in &variable {
                        function.push(Instruction::Hide(v.to_string()));
                    }

                    exit.push(function.cursor());

                    function.push(Instruction::Null);
                },
                Statement::Return(r)              => r.compile(block, function)?,
                _ => {}
            }
        }

        if !root {
            for v in &variable {
                function.push(Instruction::Hide(v.to_string()));
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
    pub leave: Option<Identifier>,
    pub block: Block,
}

impl Function {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Function, |token_buffer| {
            token_buffer.want(TokenKind::Function)?;

            let name = token_buffer.want_identifier()?;
            let mut enter = Vec::new();
            let mut leave = None;

            token_buffer.want(TokenKind::ParenthesisBegin)?;

            // No argument branch.
            if token_buffer.want_peek(TokenKind::ParenthesisClose) {
                token_buffer.want(TokenKind::ParenthesisClose)?;
            } else {
                Statement::parse_comma(
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
                name,
                enter,
                leave,
                block,
            })
        })
    }

    pub fn analyze(&mut self, scope: &mut Scope) -> Result<(), Error> {
        for variable in &self.enter {
            let kind = variable.analyze(scope)?;

            let definition = Definition {
                span: variable.span.clone(),
                name: variable.name.clone(),
                kind_i: Some(variable.kind.clone()),
                kind_e: Some(kind),
                value: Expression::Value(ExpressionValue::Path(Path {
                    list: vec![PathKind::Identifier(variable.name.clone())],
                })),
            };

            // TO-DO incorrect, this is setting each parameter as a global value and not inside the scope!
            // do this inside the scope.
            scope.set_declaration(variable.name.clone(), Declaration::Definition(definition));
        }

        self.block.analyze(scope, false)?;

        let flow = self.block.analyze_flow(scope, false)?;
        let target = if let Some(leave) = &self.leave {
            Variable::type_check(&self.span, leave, scope)?
        } else {
            ExpressionKind::Null
        };
        let source = flow.kind(false);

        if source != target {
            return Err(Error::new_info(
                ErrorInfo::new_point(self.span.clone(), None),
                ErrorKind::IncorrectKind(target, source),
                None,
            ));
        }

        //println!("{flow:#?}");

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

    fn type_check(
        _: &TokenSpan,
        name: &Identifier,
        scope: &Scope,
    ) -> Result<ExpressionKind, Error> {
        Ok(match name.text.as_str() {
            "String" => ExpressionKind::String,
            "Integer" => ExpressionKind::Integer,
            "Decimal" => ExpressionKind::Decimal,
            "Boolean" => ExpressionKind::Boolean,
            _ => {
                let definition = scope.get_declaration(name.clone()).unwrap();

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

    pub fn analyze(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
        Self::type_check(&self.span, &self.kind, scope)
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct ArrayD {
    pub list: Vec<Expression>,
}

impl ArrayD {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        // TO-DO use ErrorHint::Array
        token_buffer.parse(ErrorHint::Structure, |token_buffer| {
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
}

//================================================================

#[derive(Debug, Clone)]
pub struct StructureD {
    pub name: Identifier,
    pub list: HashMap<String, Assignment>,
}

impl StructureD {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Structure, |token_buffer| {
            let mut list = HashMap::new();

            let name = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::CurlyBegin)?;

            Statement::parse_comma(token_buffer, TokenKind::CurlyClose, |token_buffer| {
                let assignment = Assignment::parse_token_loose(token_buffer)?;

                let name = assignment.path.list.first().unwrap();
                let name = {
                    if let PathKind::Identifier(identifier) = name {
                        identifier.text.clone()
                    } else {
                        panic!("structure assignment: path is not an identifier")
                    }
                };

                list.insert(name, assignment);
                Ok(())
            })?;

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self { name, list })
        })
    }
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub name: Identifier,
    pub variable: BTreeMap<String, Variable>,
    pub function: BTreeMap<String, Function>,
}

impl Structure {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Structure, |token_buffer| {
            let mut variable = BTreeMap::new();
            let mut function = BTreeMap::new();

            token_buffer.want(TokenKind::Structure)?;

            let name = token_buffer.want_identifier()?;

            token_buffer.want(TokenKind::CurlyBegin)?;

            while let Some(token) = token_buffer.peek() {
                if token.class.kind() == TokenKind::CurlyClose {
                    break;
                }

                if token.class.kind() == TokenKind::Function {
                    let f = Function::parse_token(token_buffer)?;
                    function.insert(f.name.text.clone(), f);
                } else if token.class.kind() == TokenKind::Identifier {
                    let v = Variable::parse_token(token_buffer)?;
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

            //Statement::parse_comma(token_buffer, TokenKind::CurlyClose, |token_buffer| {
            //    variable.push(Variable::parse_token(token_buffer)?);
            //    Ok(())
            //})?;

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self {
                name,
                variable,
                function,
            })
        })
    }

    pub fn analyze(&mut self, scope: &mut Scope) -> Result<(), Error> {
        for variable in self.variable.values() {
            variable.analyze(scope)?;
        }

        for function in self.function.values_mut() {
            function.analyze(scope)?;
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

            Statement::parse_comma(token_buffer, TokenKind::CurlyClose, |token_buffer| {
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

    pub fn analyze(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
        println!("enter path: {:?}", self);

        let mut active = None;
        let mut result = ExpressionKind::Null;

        for path in &self.list {
            if let Some(d_active) = &active {
                match d_active {
                    Declaration::Structure(structure) => match path {
                        PathKind::Identifier(identifier) => {
                            let field = structure.variable.get(&identifier.text).unwrap();

                            let kind =
                                Variable::type_check(&field.span, &field.kind.clone(), scope)?;

                            match kind {
                                ExpressionKind::Structure(structure) => {
                                    active = Some(
                                        scope.get_declaration(structure.clone()).unwrap().clone(),
                                    );
                                    result = ExpressionKind::Structure(structure);
                                }
                                x => {
                                    result = x;
                                }
                            }
                        }
                        PathKind::Invocation(invocation) => {
                            let field = structure.function.get(&invocation.name.text).unwrap();

                            if let Some(leave) = &field.leave {
                                let kind = Variable::type_check(&field.span, leave, scope)?;

                                match kind {
                                    ExpressionKind::Structure(structure) => {
                                        active = Some(
                                            scope
                                                .get_declaration(structure.clone())
                                                .unwrap()
                                                .clone(),
                                        );
                                        result = ExpressionKind::Structure(structure);
                                    }
                                    x => {
                                        result = x;
                                    }
                                }
                            }
                        }
                        _ => panic!("analyze: invalid path: {path:#?}"),
                    },
                    x => panic!("analyze: invalid active: {x:#?}"),
                }
            } else {
                match path {
                    PathKind::Identifier(identifier) => {
                        if let Some(value) = scope.get_declaration(identifier.clone()) {
                            match value {
                                Declaration::Definition(definition) => {
                                    let kind = definition.kind_e.clone().expect(&format!(
                                        "no kind_e for def {}",
                                        definition.name.text
                                    ));

                                    // TO-DO this will crash without explicit typing.
                                    //let kind = Variable::type_check(
                                    //    &definition.span,
                                    //    &definition.kind.clone().unwrap(),
                                    //    scope,
                                    //)?;

                                    match kind {
                                        ExpressionKind::Structure(structure) => {
                                            active = Some(
                                                scope
                                                    .get_declaration(structure.clone())
                                                    .unwrap()
                                                    .clone(),
                                            );
                                            result = ExpressionKind::Structure(structure);
                                        }
                                        x => {
                                            result = x;
                                        }
                                    }
                                }
                                Declaration::Structure(_) => {
                                    active = Some(value.clone());
                                }
                                _ => todo!(),
                            }
                        } else {
                            panic!("analyze: unknown symbol: {}", identifier.text);
                        }
                    }
                    PathKind::Invocation(invocation) => result = invocation.analyze(scope)?,
                    _ => todo!(),
                };
            }
        }

        println!("leave path: {:?}", self);

        Ok(result)
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        let mut active = None;

        for path in &self.list {
            if let Some(d_active) = &active {
                match d_active {
                    Declaration::Structure(structure) => match path {
                        PathKind::Identifier(identifier) => {
                            function.push(Instruction::LoadField(identifier.text.to_string()));

                            let field = structure.variable.get(&identifier.text).unwrap();

                            let kind =
                                Variable::type_check(&field.span, &field.kind.clone(), scope)?;

                            if let ExpressionKind::Structure(structure) = kind {
                                active =
                                    Some(scope.get_declaration(structure.clone()).unwrap().clone());
                            }
                        }
                        PathKind::Invocation(invocation) => {
                            for expression in invocation.list.iter().rev() {
                                expression.compile(scope, function)?;
                            }

                            function.push(Instruction::Call(
                                FunctionCall::FunctionStructure(
                                    structure.name.text.clone(),
                                    invocation.name.text.clone(),
                                ),
                                invocation.list.len(),
                            ));

                            let field = structure.function.get(&invocation.name.text).unwrap();

                            if let Some(leave) = &field.leave {
                                let kind = Variable::type_check(&field.span, leave, scope)?;

                                match kind {
                                    ExpressionKind::Structure(structure) => {
                                        active = Some(
                                            scope
                                                .get_declaration(structure.clone())
                                                .unwrap()
                                                .clone(),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => panic!("compile: invalid path: {path:#?}"),
                    },
                    x => panic!("compile: invalid active: {x:#?}"),
                }
            } else {
                match path {
                    PathKind::Identifier(identifier) => {
                        if let Some(value) = scope.get_declaration(identifier.clone()) {
                            match value {
                                Declaration::Definition(definition) => {
                                    function.push(Instruction::Load(identifier.text.to_string()));

                                    let kind = definition.value.analyze(scope)?;

                                    if let ExpressionKind::Structure(structure) = kind {
                                        active = Some(
                                            scope
                                                .get_declaration(structure.clone())
                                                .unwrap()
                                                .clone(),
                                        );
                                    }
                                }
                                Declaration::Structure(_) => {
                                    active = Some(value.clone());
                                }
                                _ => todo!(),
                            }
                        } else {
                            println!("compile: {:#?}", scope.symbol);
                            panic!("compile: unknown symbol: {}", identifier.text);
                        }
                    }
                    PathKind::Invocation(invocation) => {
                        for expression in invocation.list.iter().rev() {
                            expression.compile(scope, function)?;
                        }

                        function.push(Instruction::Call(
                            FunctionCall::Function(invocation.name.text.to_string()),
                            invocation.list.len(),
                        ));

                        //active = invocation.analyze(scope)?;
                    }
                    _ => todo!(),
                };
            }
        }

        Ok(())
    }
}
