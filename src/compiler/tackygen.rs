use crate::{
    storage::{
        ast::{AstExpression, AstFunctionDefinition, AstProgram, AstStatement, AstUnaryOp},
        tacky::{FunctionDefinition, Instruction, Program, UnaryOp, Val},
    },
};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::storage::ast::AstBinaryOp;
use crate::storage::tacky::BinaryOp;

pub fn emit_tacky(program: AstProgram) -> Program {
    match program {
        AstProgram::Program(program) => {
            Program::Program(emit_tacky_function(program))
        }
    }
}

fn emit_tacky_function(function: AstFunctionDefinition) -> FunctionDefinition {
    match function {
        AstFunctionDefinition::Function(name, statement) => {
            FunctionDefinition::Function(name, emit_tacky_statement(statement))
        }
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
        AstExpression::Constant(num) => (vec![], Val::Constant(num)),
        AstExpression::Unary(unary_op, inner_exp) => {
            let (mut inner_instructions, v) = emit_tacky_expression(*inner_exp);

            let dst_name = format!("tmp.{}", COUNTER.fetch_add(1, Ordering::Relaxed));

            let dst = Val::Var(dst_name);
            let tacky_op = convert_unary_op(unary_op);
            inner_instructions.push(Instruction::Unary(tacky_op, v, dst.clone()));
            (inner_instructions, dst)
        }
        AstExpression::Binary(bin_op, left, right) => {
            let (mut left_instructions ,v1) = emit_tacky_expression(*left);
            let (mut right_instructions, v2) = emit_tacky_expression(*right);
            let dst_name = format!("tmp.{}", COUNTER.fetch_add(1, Ordering::Relaxed));
            let dst = Val::Var(dst_name);
            let tacky_op = convert_binary_op(bin_op);

            left_instructions.append(&mut right_instructions);
            left_instructions.push(Instruction::Binary(tacky_op, v1, v2, dst.clone()));
            (left_instructions, dst)
        }
    }
}

fn convert_unary_op(unary_op: AstUnaryOp) -> UnaryOp {
    match unary_op {
        AstUnaryOp::Complement => UnaryOp::Complement,
        AstUnaryOp::Negate => UnaryOp::Negate,
        _ => todo!()
    }
}

fn convert_binary_op(binary_op: AstBinaryOp) -> BinaryOp {
    match binary_op {
        AstBinaryOp::Add => BinaryOp::Add,
        AstBinaryOp::Subtract => BinaryOp::Subtract,
        AstBinaryOp::Multiply => BinaryOp::Multiply,
        AstBinaryOp::Divide => BinaryOp::Divide,
        AstBinaryOp::Remainder => BinaryOp::Remainder,
        _ => todo!()
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
