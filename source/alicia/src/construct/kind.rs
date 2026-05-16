use super::expression::*;
use super::statement::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::helper::*;
use crate::scope::*;
use crate::token::*;

//================================================================

#[derive(Debug, Clone)]
pub struct Kind {
    pub name: Identifier,
    pub list: Vec<Self>,
    pub reference: bool,
}

impl Kind {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
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

    pub fn type_check(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
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
            "Tuple" => {
                let mut list = Vec::new();

                for k in &self.list {
                    list.push(k.type_check(scope)?)
                }

                ExpressionKind::Tuple(list)
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
