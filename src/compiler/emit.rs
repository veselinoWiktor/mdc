use crate::storage::assembly::{AssemblyBinaryOp, AssemblyFunctionDefinition, AssemblyInstruction, AssemblyOperand, AssemblyProgram, AssemblyRegister, AssemblyUnaryOp};

pub fn emit_assembly(program: AssemblyProgram) -> String {
    let mut result = String::new();
    match program
    {
        AssemblyProgram::Program(function) => {
            result.push_str(emit_function(function).as_str())
        },
    }
    result.push_str("\t.section .note.GNU-stack,\"\",@progbits\n");

    result
}

fn emit_function(function: AssemblyFunctionDefinition) -> String {
    let mut result = String::new();
    match function {
        AssemblyFunctionDefinition::Function(identifier, instructions) => {
            result.push_str(format!("\t.global {}\n", identifier).as_str());
            result.push_str(format!("{}:\n", identifier).as_str());

            // create assembly stack
            result.push_str("\tpushq\t%rbp\n");
            result.push_str("\tmovq\t%rsp, %rbp\n");
            for instruction in instructions {
                result.push_str(emit_instruction(instruction).as_str());
            }
        }
    }
    result
}

fn emit_instruction(instruction: AssemblyInstruction) -> String {
    let mut result = String::new();

    match instruction
    {
        AssemblyInstruction::Mov(src, dest) => {
            result.push_str(format!("\tmovl\t{}, {}\n", emit_operand(src), emit_operand(dest)).as_str());
        }
        AssemblyInstruction::Ret => {
            result.push_str("\tmovq\t%rbp, %rsp\n");
            result.push_str("\tpopq\t%rbp\n");
            result.push_str("\tret\n");
        }
        AssemblyInstruction::Unary(unary_op, operand ) => {
            result.push_str(format!("\t{}\t{}\n", emit_unary_op(unary_op), emit_operand(operand)).as_str());
        }
        AssemblyInstruction::Binary(binary_op, src, dst) => {
            result.push_str(format!("\t{}\t{}, {}\n", emit_binary_op(binary_op), emit_operand(src), emit_operand(dst)).as_str());
        }
        AssemblyInstruction::Idiv(operand) => {
            result.push_str(format!("\tidivl\t{}\n", emit_operand(operand)).as_str());
        }
        AssemblyInstruction::Cdq => {
            result.push_str("\tcdq\n");
        }
        AssemblyInstruction::AllocateStack(num) => {
            result.push_str(format!("\tsubq\t${}, %rsp\n", num).as_str());
        }
    }

    result
}

fn emit_unary_op(un_op: AssemblyUnaryOp) -> String {
    match un_op {
        AssemblyUnaryOp::Neg => "negl".to_string(),
        AssemblyUnaryOp::Not => "notl".to_string()
    }
}

fn emit_binary_op(bin_op: AssemblyBinaryOp) -> String {
    match bin_op {
        AssemblyBinaryOp::Add => "addl".to_string(),
        AssemblyBinaryOp::Sub => "subl".to_string(),
        AssemblyBinaryOp::Mult => "imull".to_string(),
    }
}

fn emit_operand(operand: AssemblyOperand) -> String {
    match operand {
        AssemblyOperand::Reg(AssemblyRegister::AX) => "%eax".to_string(),
        AssemblyOperand::Reg(AssemblyRegister::DX) => "%edx".to_string(),
        AssemblyOperand::Reg(AssemblyRegister::R10) => "%r10d".to_string(),
        AssemblyOperand::Reg(AssemblyRegister::R11) => "%r11d".to_string(),
        AssemblyOperand::Stack(num) => format!("{}(%rbp)", num),
        AssemblyOperand::Imm(num) => format!("${}", num),
        _ => unreachable!()
    }
}