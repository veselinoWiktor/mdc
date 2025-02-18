use crate::{
    storage::{
        ast::{AstExpression, AstFunctionDefinition, AstProgram, AstStatement, AstUnaryOp},
        tacky::{FunctionDefinition, Instruction, Program, UnaryOp, Val},
    },
};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::storage::ast::{AstBinaryOp, AstBlockItem, AstDeclaration};
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
        AstFunctionDefinition::Function(name, body) => {
            let mut result_instructions = vec![];

            for block_item in body {
                match block_item {
                    AstBlockItem::Declaration(declaration) => {
                        result_instructions.append(&mut emit_tacky_declaration(declaration));
                    },
                    AstBlockItem::Statement(statement) => {
                        result_instructions.append(&mut emit_tacky_statement(statement));
                    }
                }
            }

            result_instructions.push(Instruction::Return(Val::Constant(0)));

            FunctionDefinition::Function(name, result_instructions)
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
        AstStatement::Expression(expression) => {
            let (instructions, _var) = emit_tacky_expression(expression);
            instructions
        }
        AstStatement::Null => vec![]
    }
}

fn emit_tacky_declaration(declaration: AstDeclaration) -> Vec<Instruction> {
    match declaration {
        AstDeclaration::Declaration(identifier, init) => {
            if let None = init {
                vec![]
            } else {
                let (mut instructions, var) = emit_tacky_expression(init.unwrap());

                instructions.push(Instruction::Copy(var, Val::Var(identifier)));
                instructions
            }
        }
    }
}

fn emit_tacky_expression(expression: AstExpression) -> (Vec<Instruction>, Val) {
    pub static VAR_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static AND_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static OR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    match expression {
        AstExpression::Constant(num) => (vec![], Val::Constant(num)),
        AstExpression::Unary(unary_op, inner_exp) => {
            let (mut inner_instructions, v) = emit_tacky_expression(*inner_exp);

            let dst_name = format!("tmp.{}", VAR_COUNTER.fetch_add(1, Ordering::Relaxed));

            let dst = Val::Var(dst_name);
            let tacky_op = convert_unary_op(unary_op);
            inner_instructions.push(Instruction::Unary(tacky_op, v, dst.clone()));
            (inner_instructions, dst)
        }
        AstExpression::Binary(AstBinaryOp::And, left, right) => {
            let label_counter = AND_COUNTER.fetch_add(1, Ordering::Relaxed);

            let jump_name = format!("and_false{}", label_counter);

            let (mut left_instructions ,v1) = emit_tacky_expression(*left);
            left_instructions.push(Instruction::JumpIfZero(v1, jump_name.clone()));
            let (mut right_instructions ,v2) = emit_tacky_expression(*right);
            right_instructions.push(Instruction::JumpIfZero(v2, jump_name.clone()));

            left_instructions.append(&mut right_instructions);

            let res_name = format!("tmp.{}", VAR_COUNTER.fetch_add(1, Ordering::Relaxed));
            let res = Val::Var(res_name);

            left_instructions.push(Instruction::Copy(Val::Constant(1), res.clone()));
            left_instructions.push(Instruction::Jump(format!("and_false_end{}", label_counter).to_string()));
            left_instructions.push(Instruction::Label(jump_name));
            left_instructions.push(Instruction::Copy(Val::Constant(0), res.clone()));
            left_instructions.push(Instruction::Label(format!("and_false_end{}", label_counter).to_string()));

            (left_instructions, res)
        },
        AstExpression::Binary(AstBinaryOp::Or, left, right) => {
            let label_counter = OR_COUNTER.fetch_add(1, Ordering::Relaxed);

            let jump_name = format!("or_false{}",label_counter);

            let (mut left_instructions ,v1) = emit_tacky_expression(*left);
            left_instructions.push(Instruction::JumpIfNotZero(v1, jump_name.clone()));
            let (mut right_instructions ,v2) = emit_tacky_expression(*right);
            right_instructions.push(Instruction::JumpIfNotZero(v2, jump_name.clone()));

            left_instructions.append(&mut right_instructions);

            let res_name = format!("tmp.{}", VAR_COUNTER.fetch_add(1, Ordering::Relaxed));
            let res = Val::Var(res_name);

            left_instructions.push(Instruction::Copy(Val::Constant(0), res.clone()));
            left_instructions.push(Instruction::Jump(format!("or_false_end{}", label_counter).to_string()));
            left_instructions.push(Instruction::Label(jump_name.clone()));
            left_instructions.push(Instruction::Copy(Val::Constant(1), res.clone()));
            left_instructions.push(Instruction::Label(format!("or_false_end{}", label_counter).to_string()));

            (left_instructions, res)
        },
        AstExpression::Binary(bin_op, left, right) => {
            let (mut left_instructions ,v1) = emit_tacky_expression(*left);
            let (mut right_instructions, v2) = emit_tacky_expression(*right);
            let dst_name = format!("tmp.{}", VAR_COUNTER.fetch_add(1, Ordering::Relaxed));
            let dst = Val::Var(dst_name);
            let tacky_op = convert_binary_op(bin_op);

            left_instructions.append(&mut right_instructions);
            left_instructions.push(Instruction::Binary(tacky_op, v1, v2, dst.clone()));
            (left_instructions, dst)
        },
        AstExpression::Var(identifier) => (vec![], Val::Var(identifier)),
        AstExpression::Assignment(var , rhs) => {
            let (mut instructions, result) = emit_tacky_expression(*rhs);

            match *var {
                AstExpression::Var(var_name) => {
                    instructions.push(Instruction::Copy(result, Val::Var(var_name.clone())));
                    (instructions, Val::Var(var_name.clone()))
                }
                _ => unreachable!()
            }
        }
    }
}

fn convert_unary_op(unary_op: AstUnaryOp) -> UnaryOp {
    match unary_op {
        AstUnaryOp::Complement => UnaryOp::Complement,
        AstUnaryOp::Negate => UnaryOp::Negate,
        AstUnaryOp::Not => UnaryOp::Not,
    }
}

fn convert_binary_op(binary_op: AstBinaryOp) -> BinaryOp {
    match binary_op {
        AstBinaryOp::Add => BinaryOp::Add,
        AstBinaryOp::Subtract => BinaryOp::Subtract,
        AstBinaryOp::Multiply => BinaryOp::Multiply,
        AstBinaryOp::Divide => BinaryOp::Divide,
        AstBinaryOp::Remainder => BinaryOp::Remainder,
        AstBinaryOp::Equal => BinaryOp::Equal,
        AstBinaryOp::NotEqual => BinaryOp::NotEqual,
        AstBinaryOp::LessThan => BinaryOp::LessThan,
        AstBinaryOp::LessOrEqual => BinaryOp::LessOrEqual,
        AstBinaryOp::GreaterThan => BinaryOp::GreaterThan,
        AstBinaryOp::GreaterOrEqual => BinaryOp::GreaterOrEqual,
        _ => unreachable!() // can't reach And | Or because they are handled earlier
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::tackygen::{convert_binary_op, convert_unary_op, emit_tacky_expression};
    use crate::storage::ast::{AstBinaryOp, AstExpression, AstUnaryOp};
    use crate::storage::tacky::{BinaryOp, Instruction, UnaryOp, Val};

    #[test]
    fn convert_binary_op_test() {
        let mut ast_bin_ops = vec![AstBinaryOp::Add, AstBinaryOp::Subtract, AstBinaryOp::Multiply,
             AstBinaryOp::Divide, AstBinaryOp::Remainder, AstBinaryOp::Equal,
             AstBinaryOp::NotEqual, AstBinaryOp::LessThan, AstBinaryOp::LessOrEqual,
            AstBinaryOp::GreaterThan, AstBinaryOp::GreaterOrEqual];

        assert_eq!(convert_binary_op(ast_bin_ops.pop().unwrap()), BinaryOp::GreaterOrEqual);
        assert_eq!(convert_binary_op(ast_bin_ops.pop().unwrap()), BinaryOp::GreaterThan);
        assert_eq!(convert_binary_op(ast_bin_ops.pop().unwrap()), BinaryOp::LessOrEqual);
        assert_eq!(convert_binary_op(ast_bin_ops.pop().unwrap()), BinaryOp::LessThan);
        assert_eq!(convert_binary_op(ast_bin_ops.pop().unwrap()), BinaryOp::NotEqual);
        assert_eq!(convert_binary_op(ast_bin_ops.pop().unwrap()), BinaryOp::Equal);
        assert_eq!(convert_binary_op(ast_bin_ops.pop().unwrap()), BinaryOp::Remainder);
        assert_eq!(convert_binary_op(ast_bin_ops.pop().unwrap()), BinaryOp::Divide);
        assert_eq!(convert_binary_op(ast_bin_ops.pop().unwrap()), BinaryOp::Multiply);
        assert_eq!(convert_binary_op(ast_bin_ops.pop().unwrap()), BinaryOp::Subtract);
        assert_eq!(convert_binary_op(ast_bin_ops.pop().unwrap()), BinaryOp::Add);
    }

    #[test]
    fn convert_unary_op_test() {
        let mut ast_un_ops = vec![AstUnaryOp::Complement, AstUnaryOp::Negate, AstUnaryOp::Not];

        assert_eq!(convert_unary_op(ast_un_ops.pop().unwrap()), UnaryOp::Not);
        assert_eq!(convert_unary_op(ast_un_ops.pop().unwrap()), UnaryOp::Negate);
        assert_eq!(convert_unary_op(ast_un_ops.pop().unwrap()), UnaryOp::Complement);
    }

    #[test]
    fn convert_constant_tacky_expression_test() {
        let expr = AstExpression::Constant(2);

        let (tacky_instructions, val) = emit_tacky_expression(expr);

        assert_eq!(tacky_instructions.len(), 0);
        assert_eq!(val, Val::Constant(2));
    }

    #[test]
    fn convert_simple_unary_tacky_expression_test() {
        let expr = AstExpression::Unary(AstUnaryOp::Negate, Box::new(AstExpression::Constant(2)));

        let (tacky_instructions, val) = emit_tacky_expression(expr);

        assert_eq!(tacky_instructions.len(), 1);
        assert_eq!(tacky_instructions[0], Instruction::Unary(UnaryOp::Negate, Val::Constant(2), val));
    }


    #[test]
    fn convert_simple_binary_tacky_expression_test() {
        let expr = AstExpression::Binary(AstBinaryOp::Divide, Box::new(AstExpression::Constant(2)), Box::new(AstExpression::Constant(1)));


        let (tacky_instructions, val) = emit_tacky_expression(expr);


        assert_eq!(tacky_instructions.len(), 1);
        assert_eq!(tacky_instructions[0], Instruction::Binary(BinaryOp::Divide, Val::Constant(2), Val::Constant(1), val));
    }

    #[test]
    fn convert_var_tacky_expression_test() {
        let expr = AstExpression::Var("some_identifier".to_string());

        let (tacky_instructions, val) = emit_tacky_expression(expr);
        assert_eq!(tacky_instructions.len(), 0);
        assert_eq!(val, Val::Var("some_identifier".to_string()));
    }
}
