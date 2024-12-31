use crate::compiler::token::Token;
use crate::storage::ast::{AstExpression, AstFunctionDefinition, AstProgram, AstStatement};
use crate::storage::assembly::{AssemblyFunctionDefinition, AssemblyInstruction, AssemblyOperand, AssemblyProgram};

pub fn gen(program: AstProgram) -> AssemblyProgram {
    match program {
        AstProgram::ProgramNode(function) => {
            AssemblyProgram::Program(convert_function(function))
        }
    }
}

fn convert_function(function: AstFunctionDefinition) -> AssemblyFunctionDefinition {
    match function {
        AstFunctionDefinition::FunctionNode(Token::Identifier(name), statement) => {
            AssemblyFunctionDefinition::Function(name, convert_statement(statement))
        }
        _ => unreachable!()
    }
}

fn convert_statement (statement: AstStatement) -> Vec<AssemblyInstruction>
{
    match statement {
        AstStatement::ReturnNode(expr) => {
            let v = convert_exp(expr);
            vec![AssemblyInstruction::Mov(v, AssemblyOperand::Register()), AssemblyInstruction::Ret]
        }
    }
}

fn convert_exp(expr: AstExpression) -> AssemblyOperand
{
    match expr {
        AstExpression::ConstantNode(Token::Constant(num)) => {
            AssemblyOperand::Imm(num)
        }
        _ => unreachable!()
    }
}