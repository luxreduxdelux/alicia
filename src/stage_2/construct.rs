use crate::helper::error::*;
use crate::stage_1::buffer::*;
use crate::stage_1::helper::*;
use crate::stage_1::token::*;
use crate::stage_2::scope::*;
use crate::stage_4::buffer::ArgumentBuffer;

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
}

#[derive(PartialEq, Eq)]
pub enum ExpressionKind {
    Path,
    String,
    Integer,
    Decimal,
    Boolean,
}

#[derive(Debug, Clone)]
pub enum ExpressionValue {
    Path(Path),
    String(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
}

impl ExpressionValue {
    fn from_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        if let Some(token) = token_buffer.peek_value() {
            match token.class {
                TokenClass::Identifier(_value) => Ok(Self::Path(Path::parse_token(token_buffer)?)),
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

    fn as_integer(&self) -> i64 {
        match self {
            Self::Integer(value) => *value,
            _ => panic!("value is not an integer"),
        }
    }

    fn as_decimal(&self) -> f64 {
        match self {
            Self::Decimal(value) => *value,
            _ => panic!("value is not a decimal"),
        }
    }

    #[rustfmt::skip]
    fn kind(&self) -> ExpressionKind {
        match self {
            Self::Path(_)       => ExpressionKind::Path,
            Self::String(_)     => ExpressionKind::String,
            Self::Integer(_)    => ExpressionKind::Integer,
            Self::Decimal(_)    => ExpressionKind::Decimal,
            Self::Boolean(_)    => ExpressionKind::Boolean,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl ExpressionOperator {
    #[rustfmt::skip]
    fn from_token(token: Token) -> Self {
        match token.class.kind() {
            TokenKind::Add      => Self::Add,
            TokenKind::Subtract => Self::Subtract,
            TokenKind::Multiply => Self::Multiply,
            TokenKind::Divide   => Self::Divide,
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
        }
    }

    #[rustfmt::skip]
    fn bind_power(&self) -> (f32, f32) {
        match self {
            ExpressionOperator::Add      => (1.0, 1.1),
            ExpressionOperator::Subtract => (1.0, 1.1),
            ExpressionOperator::Multiply => (2.0, 2.1),
            ExpressionOperator::Divide   => (2.0, 2.1),
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

    pub fn evaluate(&self, scope: &Scope) -> Result<ExpressionValue, Error> {
        Ok(match self {
            Expression::Value(expression_value) => expression_value.clone(),
            Expression::Operation(operator, a, b) => {
                let a = a.evaluate(scope)?;
                let b = b.evaluate(scope)?;
                let kind_a = a.kind();
                let kind_b = b.kind();

                if kind_a == kind_b {
                    match kind_a {
                        ExpressionKind::Path => todo!(),
                        ExpressionKind::String => todo!(),
                        ExpressionKind::Integer => {
                            let a = a.as_integer();
                            let b = b.as_integer();

                            match operator {
                                ExpressionOperator::Add => ExpressionValue::Integer(a + b),
                                ExpressionOperator::Subtract => ExpressionValue::Integer(a - b),
                                ExpressionOperator::Multiply => ExpressionValue::Integer(a * b),
                                ExpressionOperator::Divide => ExpressionValue::Integer(a / b),
                            }
                        }
                        ExpressionKind::Decimal => {
                            let a = a.as_decimal();
                            let b = b.as_decimal();

                            match operator {
                                ExpressionOperator::Add => ExpressionValue::Decimal(a + b),
                                ExpressionOperator::Subtract => ExpressionValue::Decimal(a - b),
                                ExpressionOperator::Multiply => ExpressionValue::Decimal(a * b),
                                ExpressionOperator::Divide => ExpressionValue::Decimal(a / b),
                            }
                        }
                        ExpressionKind::Boolean => todo!(),
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

            println!("{value:#?}");

            //println!("{:?}", value.evaluate());

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
        };

        self.value.analyze(scope)?;

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

                token_buffer.print_state();

                if let Some(token) = token_buffer.peek()
                    && token.class.kind() == TokenKind::Comma
                {
                    token_buffer.next();
                } else {
                    break;
                }
            }

            token_buffer.want(TokenKind::ParenthesisClose)?;

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

    pub fn execute(&self, scope: &Scope) -> Result<(), Error> {
        if let Some(declaration) = scope.get_declaration(self.name.clone()) {
            match declaration {
                Declaration::Function(function) => {
                    // TO-DO type check and validate the function is receiving every argument
                    for expression in &self.list {
                        expression.analyze(scope)?;
                    }

                    function.execute(scope);

                    Ok(())
                }
                Declaration::FunctionNative(function) => {
                    // TO-DO type check and validate the function is receiving every argument
                    for expression in &self.list {
                        expression.analyze(scope)?;
                    }

                    function(ArgumentBuffer::new(self.list.clone(), scope)?);

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

    pub fn execute(&self, scope: &Scope, iteration: bool) -> Result<(), Error> {
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
                //Instruction::Definition(definition) => definition.analyze(&mut scope_block)?,
                //Instruction::Assignment(assignment) => assignment.analyze(&scope_block)?,
                Instruction::Invocation(invocation) => invocation.execute(&scope_block)?,
                /*
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

    pub fn execute(&self, scope: &Scope) -> Result<(), Error> {
        self.block.execute(scope, false)?;

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
        token_buffer.print_state();

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
