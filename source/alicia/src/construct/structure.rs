use super::expression::*;
use super::function::*;
use super::statement::*;
use super::variable::*;

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

    pub fn analyze(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
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
    pub index_variable: BTreeMap<String, usize>,
}

impl Structure {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Structure, |token_buffer| {
            let mut variable = BTreeMap::new();
            let mut function = BTreeMap::new();
            let mut index_variable = BTreeMap::default();

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
