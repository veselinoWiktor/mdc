//! Converting TACKY to assembly

use crate::storage::assembly::{AssemblyBinaryOp, AssemblyCondition, AssemblyFunctionDefinition, AssemblyInstruction, AssemblyOperand, AssemblyProgram, AssemblyRegister, AssemblyUnaryOp};
use crate::storage::tacky::{BinaryOp, FunctionDefinition, Instruction, Program, UnaryOp, Val};

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
        Instruction::Unary(UnaryOp::Not, src, dst) => {
            vec![AssemblyInstruction::Cmp(AssemblyOperand::Imm(0), convert_operand(src)),
                 AssemblyInstruction::Mov(AssemblyOperand::Imm(0), convert_operand(dst.clone())),
                 AssemblyInstruction::SetCC(AssemblyCondition::E, convert_operand(dst))]
        }
        Instruction::Unary(un_op, src, dst) => {
            vec![AssemblyInstruction::Mov(convert_operand(src), convert_operand(dst.clone())),
                 AssemblyInstruction::Unary(convert_unary_op(un_op), convert_operand(dst))]
        },
        Instruction::Binary(bin_op @ (BinaryOp::Divide | BinaryOp::Remainder), src1, src2, dst) => {
            let mut result = vec![AssemblyInstruction::Mov(convert_operand(src1), AssemblyOperand::Reg(AssemblyRegister::AX)),
                 AssemblyInstruction::Cdq,
                 AssemblyInstruction::Idiv(convert_operand(src2))];

            match bin_op {
                BinaryOp::Divide => result.push(AssemblyInstruction::Mov(AssemblyOperand::Reg(AssemblyRegister::AX), convert_operand(dst))),
                BinaryOp::Remainder => result.push(AssemblyInstruction::Mov(AssemblyOperand::Reg(AssemblyRegister::DX), convert_operand(dst))),
                _ => unreachable!()
            }

            result
        },
        Instruction::Binary(bin_op @ (BinaryOp::GreaterThan | BinaryOp::GreaterOrEqual | BinaryOp::LessThan | BinaryOp::LessOrEqual | BinaryOp::Equal | BinaryOp::NotEqual), src1, src2, dst) => {
            let mut result = vec![AssemblyInstruction::Cmp(convert_operand(src2), convert_operand(src1)),
                 AssemblyInstruction::Mov(AssemblyOperand::Imm(0), convert_operand(dst.clone()))];

            match bin_op {
                BinaryOp::GreaterThan => result.push(AssemblyInstruction::SetCC(AssemblyCondition::G, convert_operand(dst))),
                BinaryOp::GreaterOrEqual => result.push(AssemblyInstruction::SetCC(AssemblyCondition::GE, convert_operand(dst))),
                BinaryOp::LessThan => result.push(AssemblyInstruction::SetCC(AssemblyCondition::L, convert_operand(dst))),
                BinaryOp::LessOrEqual => result.push(AssemblyInstruction::SetCC(AssemblyCondition::LE, convert_operand(dst))),
                BinaryOp::Equal => result.push(AssemblyInstruction::SetCC(AssemblyCondition::E, convert_operand(dst))),
                BinaryOp::NotEqual => result.push(AssemblyInstruction::SetCC(AssemblyCondition::NE, convert_operand(dst))),
                _ => unreachable!(),
            }

            result
        },
        Instruction::Binary(bin_op, src1, src2, dst) => {
            vec![AssemblyInstruction::Mov(convert_operand(src1), convert_operand(dst.clone())),
                 AssemblyInstruction::Binary(convert_binary_op(bin_op), convert_operand(src2), convert_operand(dst))]
        },
        Instruction::JumpIfZero(val, target) => {
            vec![AssemblyInstruction::Cmp(AssemblyOperand::Imm(0), convert_operand(val)),
                 AssemblyInstruction::JmpCC(AssemblyCondition::E, target)]
        },
        Instruction::JumpIfNotZero(val, target) => {
            vec![AssemblyInstruction::Cmp(AssemblyOperand::Imm(0), convert_operand(val)),
                 AssemblyInstruction::JmpCC(AssemblyCondition::NE, target)]
        },
        Instruction::Jump(target) => {
            vec![AssemblyInstruction::Jmp(target)]
        }
        Instruction::Copy(src, dst) => {
            vec![AssemblyInstruction::Mov(convert_operand(src), convert_operand(dst))]
        },
        Instruction::Label(identifier) => {
            vec![AssemblyInstruction::Label(identifier)]
        }
    }
}

fn convert_unary_op(un_op: UnaryOp) -> AssemblyUnaryOp {
    match un_op {
        UnaryOp::Complement => AssemblyUnaryOp::Not,
        UnaryOp::Negate => AssemblyUnaryOp::Neg,
        UnaryOp::Not => AssemblyUnaryOp::Not,
    }
}

fn convert_binary_op(bin_op: BinaryOp) -> AssemblyBinaryOp {
    match bin_op {
        BinaryOp::Add => AssemblyBinaryOp::Add,
        BinaryOp::Subtract => AssemblyBinaryOp::Sub,
        BinaryOp::Multiply => AssemblyBinaryOp::Mult,
        _ => unreachable!()
    }
}

fn convert_operand(operator: Val) -> AssemblyOperand {
    match operator {
        Val::Constant(num) => AssemblyOperand::Imm(num),
        Val::Var(name) => AssemblyOperand::PseudoReg(name)
    }
}