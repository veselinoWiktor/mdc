use crate::storage::ast::{AstBinaryOp, AstExpression, AstFunctionDefinition, AstProgram, AstStatement, AstUnaryOp};
use crate::ui::{ASTCanvas, ASTNode};

pub(crate) fn convert_into_ast_canvas(ast: &AstProgram) -> ASTCanvas {
    let mut root = ASTNode::new("Program(function_definition)".to_string());

    match ast {
        AstProgram::Program(function) => {
            root.children.push(convert_ast_function(function))
        }
    }

    ASTCanvas::new(root)
}

fn convert_ast_function(ast_function: &AstFunctionDefinition) -> ASTNode {
    match ast_function {
        AstFunctionDefinition::Function(identifier, statement) => {
            let mut function = ASTNode::new(format!("Function('{}', body)", identifier));
            function.children.push(convert_ast_statement(statement));

            function
        }
    }
}

fn convert_ast_statement(ast_statement: &AstStatement) -> ASTNode {
    match ast_statement {
        AstStatement::Return(expr) => {
            let mut statement = ASTNode::new("Return(exp)".to_string());
            statement.children.push(convert_ast_expression(expr));

            statement
        },
    }
}

fn convert_ast_expression(ast_expression: &AstExpression) -> ASTNode {
    match ast_expression {
        AstExpression::Constant(num) => {
            ASTNode::new(format!("Constant({})", num))
        }
        AstExpression::Binary(operator, left, right) => {
            let mut binary_node = ASTNode::new("Binary(operator, left, right)".to_string());
            binary_node.children.push(convert_ast_expression(left));

            let operator_node = match operator {
                AstBinaryOp::Add => ASTNode::new("Add".into()),
                AstBinaryOp::And => ASTNode::new("And".into()),
                AstBinaryOp::Divide => ASTNode::new("Divide".into()),
                AstBinaryOp::Equal => ASTNode::new("Equal".into()),
                AstBinaryOp::GreaterOrEqual => ASTNode::new("GreaterOrEqual".into()),
                AstBinaryOp::GreaterThan => ASTNode::new("GreaterThan".into()),
                AstBinaryOp::LessOrEqual => ASTNode::new("LessOrEqual".into()),
                AstBinaryOp::LessThan => ASTNode::new("LessThan".into()),
                AstBinaryOp::Multiply => ASTNode::new("Multiply".into()),
                AstBinaryOp::NotEqual => ASTNode::new("NotEqual".into()),
                AstBinaryOp::Or => ASTNode::new("Or".into()),
                AstBinaryOp::Remainder => ASTNode::new("Remainder".into()),
                AstBinaryOp::Subtract => ASTNode::new("Subtract".into())
            };

            binary_node.children.push(operator_node);
            binary_node.children.push(convert_ast_expression(right));

            binary_node
        }
        AstExpression::Unary(operator, expr) => {
            let mut unary_node = ASTNode::new("Unary(operator, expr)".into());

            let operator_node = match operator {
                AstUnaryOp::Not => ASTNode::new("Not".into()),
                AstUnaryOp::Complement => ASTNode::new("Complement".into()),
                AstUnaryOp::Negate => ASTNode::new("Negate".into()),
            };

            unary_node.children.push(operator_node);
            unary_node.children.push(convert_ast_expression(expr));

            unary_node
        }
    }
}