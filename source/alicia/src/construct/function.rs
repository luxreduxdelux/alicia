use super::block::*;
use super::expression::*;
use super::kind::*;
use super::statement::*;
use super::variable::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::helper::*;
use crate::machine::Function as MFunction;
use crate::scope::*;
use crate::token::*;

//================================================================

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
        let mut function = MFunction::new(self.name.text.clone());

        for parameter in &self.enter {
            function.push_parameter(parameter.name.text.clone());
        }

        self.block.compile(scope, &mut function, true, None)?;

        Ok(function)
    }
}
