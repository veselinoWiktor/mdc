//! Replacing pseudo registers

use std::collections::HashMap;
use crate::storage::assembly::{AssemblyFunctionDefinition, AssemblyInstruction, AssemblyOperand, AssemblyProgram};

/// Structure to keep tack of what stack slots we've assigned so far
struct ReplacementState {
    /// Last used stack slot
    current_offset: i32,

    /// Map from pseudo register to stack slots
    offset_map: HashMap<String, i32>
}

impl ReplacementState {
    fn new() -> Self {
        ReplacementState { current_offset: 0, offset_map: HashMap::new() }
    }
}

pub fn replace_pseudos(program: AssemblyProgram) -> (AssemblyProgram, i32) {
    match program {
        AssemblyProgram::Program(function) => {
            let (fixed_def, last_stack_slot) = replace_pseudos_in_function(function);
            (AssemblyProgram::Program(fixed_def), last_stack_slot as i32)
        }
    }
}

fn replace_pseudos_in_function(function: AssemblyFunctionDefinition) -> (AssemblyFunctionDefinition, i32)
{
    match function {
        AssemblyFunctionDefinition::Function(identifier, instructions) => {
            let (final_state, final_instructions) = instructions
                .into_iter()
                .fold(
                    (ReplacementState::new(), vec![]),
                    move |(mut state, mut new_instructions), instruction| {
                    let result = replace_pseudos_in_instruction(state, instruction);
                    state = result.0;
                    new_instructions.push(result.1);
                    (state, new_instructions)
                });

            (AssemblyFunctionDefinition::Function(identifier, final_instructions), final_state.current_offset)
        }
    }
}

fn replace_pseudos_in_instruction(mut state: ReplacementState, instruction: AssemblyInstruction) -> (ReplacementState, AssemblyInstruction) {
    match instruction {
        AssemblyInstruction::Mov(src, dst ) => {
            let new_src = replace_operand(&mut state, src);
            let new_dst = replace_operand(&mut state, dst);
            (state, AssemblyInstruction::Mov(new_src, new_dst))
        }
        AssemblyInstruction::Unary(op, dst) => {
            let new_dst = replace_operand(&mut state, dst);
            (state, AssemblyInstruction::Unary(op, new_dst))
        }
        AssemblyInstruction::Binary(op, src, dst) => {
            let new_src = replace_operand(&mut state, src);
            let new_dst = replace_operand(&mut state, dst);
            (state, AssemblyInstruction::Binary(op, new_src, new_dst))
        }
        AssemblyInstruction::Idiv(src) => {
            let new_src = replace_operand(&mut state, src);
            (state, AssemblyInstruction::Idiv(new_src))
        }
        AssemblyInstruction::Ret => {
            (state, AssemblyInstruction::Ret)
        }
        AssemblyInstruction::Cdq => {
            (state, AssemblyInstruction::Cdq)
        },
        AssemblyInstruction::Cmp(src, dst) => {
            let new_src = replace_operand(&mut state, src);
            let new_dst = replace_operand(&mut state, dst);
            (state, AssemblyInstruction::Cmp(new_src, new_dst))
        },
        AssemblyInstruction::SetCC(condition, dst) => {
            let new_dst = replace_operand(&mut state, dst);
            (state, AssemblyInstruction::SetCC(condition, new_dst))
        },
        AssemblyInstruction::Label(identifier) => {
            (state, AssemblyInstruction::Label(identifier))
        },
        AssemblyInstruction::JmpCC(condition, target) => {
            (state, AssemblyInstruction::JmpCC(condition, target))
        },
        AssemblyInstruction::Jmp(target) => {
            (state, AssemblyInstruction::Jmp(target))
        },
        AssemblyInstruction::AllocateStack(_) => {
            panic!("Internal error: AllocateStack shouldn't be present at this point")
        }
    }

}

fn replace_operand(state: &mut ReplacementState, operand: AssemblyOperand) -> AssemblyOperand {
    match operand {
        AssemblyOperand::PseudoReg(name) => {
            match state.offset_map.get(&name) {
                Some(val) => {
                    AssemblyOperand::Stack(val.clone())
                }
                None => {
                    state.current_offset = state.current_offset - 4;
                    state.offset_map.insert(name, state.current_offset);

                    AssemblyOperand::Stack(state.current_offset)
                }
            }
        }
        other => other
    }
}