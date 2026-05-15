use super::expression::*;
use super::kind::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::helper::*;
use crate::scope::*;
use crate::token::*;

//================================================================

#[derive(Debug, Clone)]
pub struct Variable {
    pub span: TokenSpan,
    pub name: Identifier,
    pub kind: Kind,
}

impl Variable {
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
        parent: Option<Identifier>,
    ) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Variable, |token_buffer| {
            let (name, kind) = if token_buffer.want_peek(TokenKind::SelfLower) {
                token_buffer.want(TokenKind::SelfLower)?;

                if let Some(parent) = &parent {
                    (
                        // TO-DO use self-lower span?
                        Identifier::from_string("self".to_string(), Point::default()).unwrap(),
                        Kind {
                            name: parent.clone(),
                            list: Vec::default(),
                            reference: false,
                        },
                    )
                } else {
                    panic!("self on non-structure/enumerate")
                }
            } else {
                let name = token_buffer.want_identifier()?;
                token_buffer.want(TokenKind::Colon)?;

                (name, Kind::parse_token(token_buffer)?)
            };

            Ok(Self {
                span: token_buffer.get_span(),
                name,
                kind,
            })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
        self.kind.type_check(scope)
    }
}
