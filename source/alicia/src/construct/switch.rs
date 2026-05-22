use super::block::*;
use super::expression::*;
use super::statement::Statement;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::helper::Identifier;
use crate::machine::Function as MFunction;
use crate::scope::*;
use crate::token::*;

//================================================================

#[derive(Debug, Clone)]
pub struct SwitchBlock {
    pub kind: (Identifier, Identifier),
    pub data: Vec<Identifier>,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub struct Switch {
    pub value: Expression,
    pub branch: Vec<SwitchBlock>,
}

impl Switch {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        // TO-DO use ErrorHint::Switch
        token_buffer.parse(ErrorHint::Condition, |token_buffer| {
            token_buffer.want(TokenKind::Switch)?;

            token_buffer.want(TokenKind::ParenthesisBegin)?;
            let value = Expression::parse_token(token_buffer, 0.0)?;
            token_buffer.want(TokenKind::ParenthesisClose)?;

            let mut branch = Vec::new();

            token_buffer.want(TokenKind::CurlyBegin)?;

            Statement::parse_comma(token_buffer, TokenKind::CurlyClose, |token_buffer| {
                //
                let base = token_buffer.want_identifier()?;
                token_buffer.want(TokenKind::Colon)?;
                let kind = token_buffer.want_identifier()?;

                let mut data = Vec::new();

                token_buffer.want(TokenKind::ParenthesisBegin)?;

                Statement::parse_comma(
                    token_buffer,
                    TokenKind::ParenthesisClose,
                    |token_buffer| {
                        data.push(token_buffer.want_identifier()?);
                        Ok(())
                    },
                )?;

                token_buffer.want(TokenKind::ParenthesisClose)?;

                let block = Block::parse_token(token_buffer)?;

                branch.push(SwitchBlock {
                    kind: (base, kind),
                    data,
                    block,
                });

                Ok(())
            })?;

            token_buffer.want(TokenKind::CurlyClose)?;

            println!("value: {value:#?}, branch: {branch:#?}");

            Ok(Self { value, branch })
        })
    }

    pub fn analyze(&mut self, scope: ScopePointer) -> Result<(), Error> {
        Ok(())
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        Ok(())
    }
}
