//! Invalid instructions fix-up

use crate::storage::assembly::{AssemblyFunctionDefinition, AssemblyInstruction, AssemblyOperand, AssemblyProgram, AssemblyRegister};
use crate::storage::assembly::AssemblyInstruction::AllocateStack;

pub fn fixup_program(last_stack_slot: i32, program: AssemblyProgram) -> AssemblyProgram {
    match program {
        AssemblyProgram::Program(function) => {
            AssemblyProgram::Program(fixup_function(last_stack_slot, function))
        }
    }
}

fn fixup_function(last_stack_slot: i32, function: AssemblyFunctionDefinition) -> AssemblyFunctionDefinition {
    match function {
        AssemblyFunctionDefinition::Function(identifier, instructions) => {
            let mut fixed_instructions = instructions
                .into_iter()
                .flat_map(|i| fixup_instruction(i))
                .collect::<Vec<_>>();

            fixed_instructions.insert(0, AssemblyInstruction::AllocateStack(-last_stack_slot));
            // add allocate stack
            AssemblyFunctionDefinition::Function(identifier, fixed_instructions)
        }
    }
}

fn fixup_instruction (instruction: AssemblyInstruction) -> Vec<AssemblyInstruction> {
    match instruction {
        AssemblyInstruction::Mov(AssemblyOperand::Stack(src), AssemblyOperand::Stack(dst)) => {
            vec![AssemblyInstruction::Mov(
                     AssemblyOperand::Stack(src),
                     AssemblyOperand::Reg(AssemblyRegister::R10)),
                 AssemblyInstruction::Mov(
                     AssemblyOperand::Reg(AssemblyRegister::R10),
                     AssemblyOperand::Stack(dst))
            ]
        }
        other => vec![other],
    }
}