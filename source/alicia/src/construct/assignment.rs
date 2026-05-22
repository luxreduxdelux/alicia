use super::expression::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::machine::Function as MFunction;
use crate::scope::*;
use crate::token::*;

//================================================================

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

            let value = match kind.class.kind() {
                TokenKind::DefinitionAdd => Expression {
                    span: value.span.clone(),
                    data: ExpressionData::Operation(
                        ExpressionOperator::Add,
                        Box::new(path.clone()),
                        Box::new(value),
                    ),
                },
                TokenKind::DefinitionSubtract => Expression {
                    span: value.span.clone(),
                    data: ExpressionData::Operation(
                        ExpressionOperator::Subtract,
                        Box::new(path.clone()),
                        Box::new(value),
                    ),
                },
                TokenKind::DefinitionMultiply => Expression {
                    span: value.span.clone(),
                    data: ExpressionData::Operation(
                        ExpressionOperator::Multiply,
                        Box::new(path.clone()),
                        Box::new(value),
                    ),
                },
                TokenKind::DefinitionDivide => Expression {
                    span: value.span.clone(),
                    data: ExpressionData::Operation(
                        ExpressionOperator::Divide,
                        Box::new(path.clone()),
                        Box::new(value),
                    ),
                },
                TokenKind::DefinitionModulo => Expression {
                    span: value.span.clone(),
                    data: ExpressionData::Operation(
                        ExpressionOperator::Modulo,
                        Box::new(path.clone()),
                        Box::new(value),
                    ),
                },
                TokenKind::DefinitionAnd => Expression {
                    span: value.span.clone(),
                    data: ExpressionData::Operation(
                        ExpressionOperator::LogicalAnd,
                        Box::new(path.clone()),
                        Box::new(value),
                    ),
                },
                TokenKind::DefinitionOr => Expression {
                    span: value.span.clone(),
                    data: ExpressionData::Operation(
                        ExpressionOperator::LogicalOr,
                        Box::new(path.clone()),
                        Box::new(value),
                    ),
                },
                TokenKind::DefinitionExclusiveOr => Expression {
                    span: value.span.clone(),
                    data: ExpressionData::Operation(
                        ExpressionOperator::ExclusiveOr,
                        Box::new(path.clone()),
                        Box::new(value),
                    ),
                },
                _ => value,
            };

            //println!("assignment: {value:?}");

            Ok(Self {
                span: token_buffer.get_span(),
                path: path.clone(),
                kind,
                value,
            })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        let source = self.value.analyze(scope, None)?;
        let target = self.path.analyze(scope, None)?;

        // TO-DO highlight source code where error was made
        if source != target {
            return Error::new_info(
                ErrorInfo::new_point(self.span.clone(), None, scope.get_active_source()),
                ErrorKind::IncorrectKind(target, source),
                None,
            );
        }

        Ok(())
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        self.value.compile(scope, function)?;
        self.path.compile_l(scope, function, false)?;

        Ok(())
    }
}
