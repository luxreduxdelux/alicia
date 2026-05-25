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
pub struct Import {
    pub path: Identifier,
}

impl Import {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        // TO-DO use error hint Import
        token_buffer.parse(ErrorHint::Structure, |token_buffer| {
            token_buffer.want(TokenKind::Import)?;

            let path = token_buffer.want_identifier()?;

            Ok(Self { path })
        })
    }

    pub fn analyze(&mut self, scope: ScopePointer) -> Result<ExpressionKind, Error> {
        todo!()
    }
}
