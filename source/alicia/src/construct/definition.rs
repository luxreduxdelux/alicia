use super::expression::*;
use super::kind::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::helper::*;
use crate::machine::Function as MFunction;
use crate::machine::Instruction;
use crate::scope::*;
use crate::token::*;

//================================================================

#[derive(Debug, Clone)]
pub struct Definition {
    pub span: TokenSpan,
    pub name: Identifier,
    pub constant: bool,
    pub kind_i: Option<Kind>,
    pub kind_e: Option<ExpressionKind>,
    pub value: Expression,
    pub index: Option<usize>,
}

impl Definition {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Definition, |token_buffer| {
            token_buffer.want(TokenKind::Let)?;
            let name = token_buffer.want_identifier()?;

            let kind_i = {
                if token_buffer.want_peek(TokenKind::Colon) {
                    token_buffer.want(TokenKind::Colon)?;
                    Some(Kind::parse_token(token_buffer)?)
                } else {
                    None
                }
            };

            let constant = if token_buffer.want_peek(TokenKind::DefinitionVariable) {
                token_buffer.want(TokenKind::DefinitionVariable)?;
                false
            } else {
                token_buffer.want(TokenKind::DefinitionConstant)?;
                true
            };

            let value = Expression::parse_token(token_buffer, 0.0)?;
            token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self {
                span: token_buffer.get_span(),
                name,
                constant,
                kind_i,
                kind_e: None,
                value,
                index: None,
            })
        })
    }

    pub fn analyze(&mut self, scope: &mut Scope) -> Result<ExpressionKind, Error> {
        let infer = if let Some(kind) = &self.kind_i {
            Some(kind.type_check(scope)?)
        } else {
            None
        };

        let source = self.value.analyze(scope, infer)?;

        if let Some(kind) = &self.kind_i {
            let target = kind.type_check(scope)?;

            if source != target {
                return Error::new_info(
                    ErrorInfo::new_point(self.span.clone(), None, scope.get_active_source()),
                    ErrorKind::IncorrectKind(target, source),
                    None,
                );
            }
        }

        self.kind_e = Some(source.clone());
        self.index = Some(scope.add_index_variable());

        scope.set_declaration(self.name.clone(), Declaration::Definition(self.clone()));

        Ok(source)
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        self.value.compile(scope, function)?;
        function.push(Instruction::Save(self.index.unwrap()));

        Ok(())
    }
}
