use super::array::*;
use super::enumerate::*;
use super::statement::*;
use super::structure::*;
use super::table::*;
use super::tuple::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::helper::*;
use crate::machine::Function as MFunction;
use crate::machine::Instruction;
use crate::machine::Value;
use crate::scope::*;
use crate::token::*;

//================================================================

use std::fmt::Display;

//================================================================

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
    Tuple(Vec<ExpressionKind>),
}

#[rustfmt::skip]
impl PartialEq for ExpressionKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Structure(l0), Self::Structure(r0)) => l0.text == r0.text,
            (Self::Enumerate(l0), Self::Enumerate(r0)) => l0.text == r0.text,
            (Self::Array(l0),     Self::Array(r0)) => l0 == r0,
            // TO-DO test if an explicit table check is necessary
            //(Self::Table(l0, l1), Self::Table(r0, r1)) => l0 == r0,
            (Self::Tuple(l0),     Self::Tuple(r0)) => l0 == r0,
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
    Tuple(TupleD),
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
            Self::Tuple(x)      => x.analyze(scope, infer)?,
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

                let mut value = Vec::new();

                Statement::parse_comma(
                    token_buffer,
                    TokenKind::ParenthesisClose,
                    |token_buffer| {
                        value.push(Self::parse_token(token_buffer, 0.0)?);
                        Ok(())
                    },
                )?;

                token_buffer.want(TokenKind::ParenthesisClose)?;

                if value.len() == 1 {
                    value.first().unwrap().clone()
                } else {
                    Expression::new(
                        token_buffer,
                        ExpressionData::Value(ExpressionValue::Tuple(TupleD::new(value))),
                    )
                }
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
                                    } else {
                                        for parameter in list {
                                            parameter.analyze(scope, infer.clone())?;
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
                        let kind = expression.as_ref().unwrap().analyze(scope, None)?;

                        match value {
                            ExpressionKind::Array(expression_kind) => {
                                if kind != ExpressionKind::Integer {
                                    panic!("non-integer index for array")
                                }

                                Ok(*expression_kind)
                            }
                            ExpressionKind::Table(a, b) => {
                                if kind != *a {
                                    panic!("key mis-match for table")
                                }

                                Ok(*b)
                            }
                            ExpressionKind::Tuple(expression_kind) => {
                                let expression = expression.as_ref().unwrap();

                                if let ExpressionData::Value(value) = &expression.data
                                    && let ExpressionValue::Integer(index) = value
                                {
                                    if *index >= 0 && *index < expression_kind.len() as i64 {
                                        return Ok(expression_kind[*index as usize].clone());
                                    } else {
                                        panic!("invalid integer index for tuple")
                                    }
                                }

                                panic!("invalid integer index for tuple")
                            }
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
                ExpressionValue::Tuple(value) => {
                    for l in value.list.iter().rev() {
                        l.compile(scope, function)?;
                    }

                    function.push(Instruction::PushTuple(value.list.len()))
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

                            //println!("{} index: {:?}", identifier, f.index);

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
                            ExpressionKind::Tuple(_) => function.push(Instruction::LoadIndexTuple),
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
