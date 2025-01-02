//! Converting TACKY to assembly

use crate::storage::assembly::{AssemblyFunctionDefinition, AssemblyInstruction, AssemblyOperand, AssemblyProgram, AssemblyRegister, AssemblyUnaryOp};
use crate::storage::tacky::{FunctionDefinition, Instruction, Program, UnaryOp, Val};

pub fn gen(program: Program) -> AssemblyProgram {
    match program {
        Program::Program(function) => {
            AssemblyProgram::Program(convert_function(function))
        }
    }
}

fn convert_function(function: FunctionDefinition) -> AssemblyFunctionDefinition {
    match function {
        FunctionDefinition::Function(name, instructions) => {
            let mut res: Vec<AssemblyInstruction> = vec![];

            for instruction in instructions {
                res.append(&mut convert_instruction(instruction))
            }

            AssemblyFunctionDefinition::Function(name, res)
        }
    }
}

fn convert_instruction(instruction: Instruction) -> Vec<AssemblyInstruction>
{
    match instruction {
        Instruction::Return(val) => {
            vec![AssemblyInstruction::Mov(convert_operand(val), AssemblyOperand::Reg(AssemblyRegister::AX)),
                 AssemblyInstruction::Ret]
        }
        Instruction::Unary(un_op, src, dst) => {
            vec![AssemblyInstruction::Mov(convert_operand(src), convert_operand(dst.clone())),
                 AssemblyInstruction::Unary(convert_operator(un_op), convert_operand(dst))]
        }
    }
}

fn convert_operator(operator: UnaryOp) -> AssemblyUnaryOp {
    match operator {
        UnaryOp::Complement => AssemblyUnaryOp::Not,
        UnaryOp::Negate => AssemblyUnaryOp::Neg
    }
}

fn convert_operand(operator: Val) -> AssemblyOperand {
    match operator {
        Val::Constant(num) => AssemblyOperand::Imm(num),
        Val::Var(name) => AssemblyOperand::PseudoReg(name)
    }
}