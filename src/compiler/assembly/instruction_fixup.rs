//! Invalid instructions fix-up

use crate::storage::assembly::{AssemblyBinaryOp, AssemblyFunctionDefinition, AssemblyInstruction, AssemblyOperand, AssemblyProgram, AssemblyRegister};

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
            vec![
                AssemblyInstruction::Mov(
                     AssemblyOperand::Stack(src),
                     AssemblyOperand::Reg(AssemblyRegister::R10)),
                AssemblyInstruction::Mov(
                     AssemblyOperand::Reg(AssemblyRegister::R10),
                     AssemblyOperand::Stack(dst))
            ]
        },
        AssemblyInstruction::Idiv(operand @ AssemblyOperand::Imm(_)) => {
            vec![
                AssemblyInstruction::Mov(
                    operand,
                    AssemblyOperand::Reg(AssemblyRegister::R10)),
                AssemblyInstruction::Idiv(
                    AssemblyOperand::Reg(AssemblyRegister::R10))
            ]
        }
        AssemblyInstruction::Binary(AssemblyBinaryOp::Add, src, dst) => {
            vec![
                AssemblyInstruction::Mov(
                    src,
                    AssemblyOperand::Reg(AssemblyRegister::R10)),
                AssemblyInstruction::Binary(
                    AssemblyBinaryOp::Add,
                    AssemblyOperand::Reg(AssemblyRegister::R10),
                    dst)]
        }
        AssemblyInstruction::Binary(AssemblyBinaryOp::Sub, src, dst) => {
            vec![
                AssemblyInstruction::Mov(
                    src,
                    AssemblyOperand::Reg(AssemblyRegister::R10)),
                AssemblyInstruction::Binary(
                    AssemblyBinaryOp::Sub,
                    AssemblyOperand::Reg(AssemblyRegister::R10),
                    dst)
            ]
        }
        AssemblyInstruction::Binary(AssemblyBinaryOp::Mult, src, AssemblyOperand::Stack(stack)) => {
            vec![
                AssemblyInstruction::Mov(
                    AssemblyOperand::Stack(stack),
                    AssemblyOperand::Reg(AssemblyRegister::R11)),
                AssemblyInstruction::Binary(
                    AssemblyBinaryOp::Mult,
                    src,
                    AssemblyOperand::Reg(AssemblyRegister::R11)),
                AssemblyInstruction::Mov(
                    AssemblyOperand::Reg(AssemblyRegister::R11),
                    AssemblyOperand::Stack(stack)),
            ]
        }
        other => vec![other],
    }
}