use crate::storage::assembly::{AssemblyFunctionDefinition, AssemblyInstruction, AssemblyOperand, AssemblyProgram};

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
            for instruction in instructions {
                result.push_str(emit_instruction(instruction).as_str());
            }
        }
    }
    result
}

fn emit_instruction(instruction: AssemblyInstruction) -> String {
    match instruction
    {
        AssemblyInstruction::Mov(src, dest)
            => format!("\tmovl {}, {}\n", emit_operand(src), emit_operand(dest)),
        AssemblyInstruction::Ret
            => "\tret\n".to_string(),
        _ => todo!()
    }
}

fn emit_operand(operand: AssemblyOperand) -> String {
    match operand {
        AssemblyOperand::Reg(_) => "%eax".to_string(),
        AssemblyOperand::Imm(num) => format!("${}", num.to_string()),
        _ => todo!()
    }
}