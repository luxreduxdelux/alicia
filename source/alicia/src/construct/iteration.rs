use super::assignment::*;
use super::block::*;
use super::expression::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::helper::Identifier;
use crate::machine::Function as MFunction;
use crate::machine::Instruction;
use crate::scope::*;
use crate::token::*;

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
pub enum IterationValue {
    Iterational { lhs: Expression, rhs: Expression },
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
                token_buffer.want(TokenKind::ParenthesisBegin)?;

                let value = if let Some(token) = token_buffer.peek_ahead(1)
                    && token.class.kind() == TokenKind::Definition
                {
                    let lhs = Expression::parse_token(token_buffer, 0.0)?;
                    token_buffer.want(TokenKind::Definition)?;
                    let rhs = Expression::parse_token(token_buffer, 0.0)?;

                    IterationValue::Iterational { lhs, rhs }
                } else {
                    let value = Expression::parse_token(token_buffer, 0.0)?;

                    IterationValue::Conditional(value)
                };

                token_buffer.want(TokenKind::ParenthesisClose)?;

                Some(value)
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
                IterationValue::Iterational { lhs, rhs } => {
                    lhs.analyze_identifier()?;
                    rhs.analyze(&scope.borrow(), None)?;
                }
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
                IterationValue::Iterational { lhs, rhs } => todo!(),
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
