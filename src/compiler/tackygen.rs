use crate::{
    compiler::token::Token,
    storage::{
        ast::{AstExpression, AstFunctionDefinition, AstProgram, AstStatement, AstUnaryOp},
        tacky::{FunctionDefinition, Instruction, Program, UnaryOp, Val},
    },
};
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn emit_tacky(program: AstProgram) -> Program {
    match program {
        AstProgram::Program(program) => {
            Program::Program(emit_tacky_function(program))
        }
    }
}

fn emit_tacky_function(function: AstFunctionDefinition) -> FunctionDefinition {
    match function {
        AstFunctionDefinition::Function(Token::Identifier(name), statement) => {
            FunctionDefinition::Function(name, emit_tacky_statement(statement))
        }
        _ => unreachable!(),
    }
}

fn emit_tacky_statement(statement: AstStatement) -> Vec<Instruction> {
    match statement {
        AstStatement::Return(expression) => {
            let (mut instructions, var) = emit_tacky_expression(expression);
            instructions.push(Instruction::Return(var));
            instructions
        }
    }
}

fn emit_tacky_expression(expression: AstExpression) -> (Vec<Instruction>, Val) {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    match expression {
        AstExpression::Constant(Token::Constant(num)) => (vec![], Val::Constant(num)),
        AstExpression::Unary(unary_op, inner_exp) => {
            let (mut inner_instructions, v) = emit_tacky_expression(*inner_exp);

            let dst_name = format!("tmp.{}", COUNTER.fetch_add(1, Ordering::Relaxed));

            let dst = Val::Var(dst_name);
            let tacky_op = convert_unary_op(unary_op);
            inner_instructions.push(Instruction::Unary(tacky_op, v, dst.clone()));
            (inner_instructions, dst)
        }
        _ => unreachable!(),
    }
}

fn convert_unary_op(unary_op: AstUnaryOp) -> UnaryOp {
    match unary_op {
        AstUnaryOp::Complement => UnaryOp::Complement,
        AstUnaryOp::Negate => UnaryOp::Negate,
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::compiler::tackygen::{emit_tacky_statement};
//     use crate::compiler::token::Token;
//     use crate::storage::ast::{AstExpression, AstStatement, AstUnaryOp};
//     use crate::storage::tacky::{Instruction, UnaryOp, Val};
//
//     #[test]
//     fn basic_test() {
//         let res = AstStatement::Return(AstExpression::Unary(AstUnaryOp::Negate, Box::new(AstExpression::Unary(AstUnaryOp::Complement, Box::new(AstExpression::Constant(Token::Constant(100)))))));
//
//         assert_eq!(emit_tacky_statement(res), vec![AstStatement::Unary(UnaryOp::Complement, Val::Constant(100), Val::Var("tmp.0".to_string())), AstStatement::Unary(UnaryOp::Negate, Val::Var("tmp.0".to_string()), Val::Var("tmp.1".to_string())), AstStatement::Return(Val::Var("tmp.1".to_string()))]);
//     }
// }
