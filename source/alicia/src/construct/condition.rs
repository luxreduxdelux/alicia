use super::block::*;
use super::expression::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::machine::Function as MFunction;
use crate::machine::Instruction;
use crate::scope::*;
use crate::token::*;

//================================================================

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

    pub fn analyze(&mut self, scope: ScopePointer, iteration: bool) -> Result<(), Error> {
        if let Some(value) = &self.value {
            let kind = value.analyze(&mut scope.borrow_mut(), None)?;

            if kind != ExpressionKind::Boolean {
                panic!("condition expression kind is not a boolean");
            }
        }

        if let Some(child) = &mut self.child {
            child.analyze(scope.clone(), iteration)?;
        }

        self.block.analyze(scope, Vec::default(), iteration)?;

        Ok(())
    }

    pub fn analyze_flow(&self, scope: &Scope) -> Result<Vec<Flow>, Error> {
        let mut list = Vec::new();

        list.push(self.block.analyze_flow(scope, self.value.is_some())?);

        if let Some(child) = &self.child {
            list.extend(child.analyze_flow(scope)?);
        }

        Ok(list)
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        let branch = if let Some(value) = &self.value {
            value.compile(scope, function)?;

            let branch = function.cursor();

            function.push(Instruction::Branch(0));

            Some(branch)
        } else {
            None
        };

        self.block.compile(scope, function, true, None)?;

        let jump = function.cursor();

        function.push(Instruction::Jump(0));

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
