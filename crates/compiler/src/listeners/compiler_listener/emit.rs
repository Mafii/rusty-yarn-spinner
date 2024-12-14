use crate::listeners::CompilerListener;
use crate::prelude::*;
use antlr_rust::token::Token;
use yarnspinner_core::prelude::OpCode;
use yarnspinner_core::prelude::*;

impl CompilerListener<'_> {
    /// Creates a new instruction, and appends it to a node in the [`Program`].
    pub(crate) fn emit(&mut self, emit: Emit) {
        let instruction = Instruction {
            opcode: emit.op_code.into(),
            operands: emit.operands,
        };

        let current_node = self.current_node.as_mut().unwrap();
        self.current_debug_info
            .line_positions
            .insert(current_node.instructions.len(), emit.source);
        current_node.instructions.push(instruction);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Emit {
    source: Option<Position>,
    op_code: OpCode,
    operands: Vec<Operand>,
}

impl Emit {
    pub(crate) fn from_op_code(op_code: OpCode) -> Self {
        Self {
            op_code,
            source: Default::default(),
            operands: Default::default(),
        }
    }

    pub(crate) fn with_source(mut self, source: Position) -> Self {
        self.source = Some(source);
        self
    }

    pub(crate) fn with_operand(mut self, operand: impl Into<Operand>) -> Self {
        self.operands.push(operand.into());
        self
    }

    pub(crate) fn with_token(mut self, token: &(impl Token + ?Sized)) -> Self {
        self.source = Some(Position {
            line: token.get_line_as_usize().saturating_sub(1),
            character: token.get_column_as_usize(),
        });
        self
    }
}

impl From<OpCode> for Emit {
    fn from(op_code: OpCode) -> Self {
        Self::from_op_code(op_code)
    }
}
