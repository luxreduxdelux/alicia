use super::expression::*;
use super::function::*;
use super::kind::Kind;
use super::statement::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::helper::*;
use crate::scope::*;
use crate::token::*;

//================================================================

use std::collections::BTreeMap;

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

            if token_buffer.want_peek(TokenKind::ParenthesisBegin) {
                token_buffer.want(TokenKind::ParenthesisBegin)?;

                Statement::parse_comma(
                    token_buffer,
                    TokenKind::ParenthesisClose,
                    |token_buffer| {
                        list.push(Expression::parse_token(token_buffer, 0.0)?);
                        Ok(())
                    },
                )?;

                token_buffer.want(TokenKind::ParenthesisClose)?;
            }

            Ok(Self { name, kind, list })
        })
    }
}

#[derive(Debug, Clone)]
pub struct Enumerate {
    pub name: Identifier,
    pub variable: OrderMap<String, Vec<Kind>>,
    pub function: OrderMap<String, Function>,
    pub index: Option<usize>,
    pub index_variable: OrderMap<String, usize>,
}

impl Enumerate {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Enumerate, |token_buffer| {
            let mut variable = OrderMap::default();
            let mut function = OrderMap::default();
            let mut index_variable = OrderMap::default();

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
                                kind.push(Kind::parse_token(token_buffer)?);
                                Ok(())
                            },
                        )?;

                        token_buffer.want(TokenKind::ParenthesisClose)?;
                    }

                    index_variable.insert(name.text.clone(), variable.len());
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
                index_variable,
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
