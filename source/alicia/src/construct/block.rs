use super::definition::*;
use super::expression::*;
use super::statement::*;
use super::variable::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::machine::Function as MFunction;
use crate::machine::Instruction;
use crate::scope::*;
use crate::token::*;

//================================================================

use std::cell::RefCell;
use std::rc::Rc;

//================================================================

#[derive(Debug, Clone)]
pub struct Block {
    pub span: TokenSpan,
    pub code: Vec<Statement>,
    pub scope: Option<ScopePointer>,
}

impl Block {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Block, |token_buffer| {
            let mut code = Vec::new();

            token_buffer.want(TokenKind::CurlyBegin)?;

            while let Some(token) = token_buffer.peek() {
                if token.class.kind() == TokenKind::CurlyClose {
                    break;
                } else {
                    code.push(Statement::parse_token(token, token_buffer)?);
                }
            }

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self {
                span: token_buffer.get_span(),
                code,
                scope: None,
            })
        })
    }

    pub fn analyze(
        &mut self,
        scope: ScopePointer,
        argument: Vec<Variable>,
        iteration: bool,
    ) -> Result<Flow, Error> {
        let scope_block = Rc::new(RefCell::new(Scope::new(Some(scope))));

        for variable in &argument {
            let kind = variable.analyze(&scope_block.borrow())?;
            let index = scope_block.borrow_mut().add_index_variable();

            let definition = Definition {
                span: variable.span.clone(),
                name: variable.name.clone(),
                constant: false,
                kind_i: Some(variable.kind.clone()),
                kind_e: Some(kind),
                // TO-DO will cause stack overflow
                value: Expression {
                    span: variable.span.clone(),
                    data: ExpressionData::Value(ExpressionValue::Identifier(variable.name.clone())),
                },
                index: Some(index),
            };

            scope_block
                .borrow_mut()
                .set_declaration(variable.name.clone(), Declaration::Definition(definition));
        }

        for statement in &self.code {
            match statement {
                Statement::Function(function) => scope_block.borrow_mut().set_declaration(
                    function.name.clone(),
                    Declaration::Function(function.clone()),
                ),
                Statement::Structure(structure) => scope_block.borrow_mut().set_declaration(
                    structure.name.clone(),
                    Declaration::Structure(structure.clone()),
                ),
                Statement::Enumerate(enumerate) => scope_block.borrow_mut().set_declaration(
                    enumerate.name.clone(),
                    Declaration::Enumerate(enumerate.clone()),
                ),
                _ => {}
            }
        }

        for statement in &mut self.code {
            match statement {
                Statement::Definition(definition) => {
                    definition.analyze(&mut scope_block.borrow_mut())?;
                }
                Statement::Assignment(assignment) => {
                    assignment.analyze(&scope_block.borrow())?;
                }
                Statement::Expression(expression) => {
                    expression.analyze(&scope_block.borrow(), None)?;
                }
                Statement::Condition(condition) => {
                    condition.analyze(scope_block.clone(), iteration)?;
                }
                Statement::Iteration(iteration) => {
                    iteration.analyze(scope_block.clone())?;
                }
                Statement::Switch(switch) => {
                    switch.analyze(scope_block.clone())?;
                }
                Statement::Block(block) => {
                    block.analyze(scope_block.clone(), Vec::default(), iteration)?;
                }
                Statement::Skip => {
                    if !iteration {
                        // TO-DO use actual span data.
                        return Error::new_kind(ErrorKind::InvalidSkip, Some(ErrorHint::Iteration));
                    }
                }
                Statement::Exit => {
                    if !iteration {
                        // TO-DO use actual span data.
                        return Error::new_kind(ErrorKind::InvalidExit, Some(ErrorHint::Iteration));
                    }
                }
                Statement::Return(r) => {
                    r.analyze(&scope_block.borrow())?;
                }
                _ => {}
            }
        }

        let flow = self.analyze_flow(&scope_block.borrow(), false)?;

        self.scope = Some(scope_block);

        Ok(flow)
    }

    pub fn analyze_flow(&self, scope: &Scope, condition: bool) -> Result<Flow, Error> {
        let mut flow = Flow::new(self.span.clone(), condition);

        for statement in &self.code {
            match statement {
                Statement::Condition(condition) => {
                    flow.list.extend(condition.analyze_flow(scope)?);
                }
                Statement::Block(block) => {
                    flow.list.push(block.analyze_flow(scope, false)?);
                }
                Statement::Return(r) => {
                    flow.kind = Some(r.analyze(scope)?);
                }
                _ => {}
            }
        }

        Ok(flow)
    }

    #[rustfmt::skip]
    pub fn compile(&self, scope: &Scope, function: &mut MFunction, root: bool, header: Option<usize>) -> Result<(), Error> {
        let block = self.scope.as_ref().unwrap();
        let variable_a = scope.get_index_variable();
        let mut variable_b = scope.get_index_variable();
        let mut exit = Vec::new();

        for statement in &self.code {
            match statement {
                Statement::Definition(definition) => {
                    definition.compile(&block.borrow(), function)?;

                    if !root {
                        variable_b += 1;
                    }
                },
                Statement::Assignment(assignment) => { assignment.compile(&block.borrow(), function)?; },
                Statement::Expression(expression) => { expression.compile(&block.borrow(), function)?; },
                Statement::Condition(condition)   => { condition.compile(&block.borrow(), function)?;  },
                Statement::Iteration(iteration)   => { iteration.compile(&block.borrow(), function)?;  },
                Statement::Switch(switch)         => { switch.compile(&block.borrow(), function)?;     },
                Statement::Block(b)               => { b.compile(&block.borrow(), function, false, None)?; },
                Statement::Skip => if let Some(header) = header {
                    for v in variable_a..variable_b {
                        function.push(Instruction::Hide(v));
                    }

                    function.push(Instruction::Jump(header));
                },
                Statement::Exit => {
                    for v in variable_a..variable_b {
                        function.push(Instruction::Hide(v));
                    }

                    exit.push(function.cursor());

                    function.push(Instruction::Jump(0));
                },
                Statement::Return(r) => r.compile(&block.borrow(), function)?,
                _ => {}
            }
        }

        if !root {
            for v in variable_a..variable_b {
                function.push(Instruction::Hide(v));
            }
        }

        for e in exit {
            function.change(Instruction::Jump(function.cursor() + 1), e);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Flow {
    //span: TokenSpan,
    list: Vec<Flow>,
    gate: bool,
    kind: Option<ExpressionKind>,
}

impl Flow {
    fn new(span: TokenSpan, gate: bool) -> Self {
        Self {
            //span,
            list: Vec::default(),
            gate,
            kind: None,
        }
    }

    pub fn kind(&self, all: bool) -> ExpressionKind {
        let mut kind = if let Some(kind) = &self.kind {
            kind.clone()
        } else {
            ExpressionKind::Null
        };
        let all_return = self.all_return();

        for flow in &self.list {
            let flow_kind = flow.kind(all || all_return);

            if kind != flow_kind {
                if kind == ExpressionKind::Null {
                    if all_return {
                        kind = flow_kind;
                    } else {
                        if !all {
                            panic!(
                                "type mis-match in flow return kind: {kind:?} != {:?}",
                                flow_kind
                            );
                        }
                    }
                } else {
                    if flow_kind != ExpressionKind::Null {
                        panic!(
                            "type mis-match in flow return kind: {kind:?} != {:?}",
                            flow_kind
                        );
                    }
                }
            }
        }

        kind
    }

    fn all_return(&self) -> bool {
        if self.list.is_empty() {
            return !self.gate && self.kind != Some(ExpressionKind::Null);
        }

        for flow in &self.list {
            // If this flow is reachable and does NOT guarantee a return → fail
            if !flow.gate && !flow.all_return() {
                return false;
            }
        }

        true
    }
}
