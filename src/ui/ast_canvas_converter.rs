use crate::storage::ast::{AstBinaryOp, AstBlockItem, AstDeclaration, AstExpression, AstFunctionDefinition, AstProgram, AstStatement, AstUnaryOp};
use crate::ui::Node;

pub(crate) fn convert_into_ast_canvas(ast: &AstProgram) -> Node {
    let mut root = Node::new("Program(function_definition)".to_string());

    match ast {
        AstProgram::Program(function) => {
            root.children.push(convert_ast_function(function))
        }
    }

    root
}

fn convert_ast_function(ast_function: &AstFunctionDefinition) -> Node {
    match ast_function {
        AstFunctionDefinition::Function(identifier, body) => {
            let mut function = Node::new(format!("Function('{}', body)", identifier));

            for block_item in body {
                match block_item {
                    AstBlockItem::Declaration(declaration) => {
                        function.children.push(convert_ast_declaration(declaration))
                    },
                    AstBlockItem::Statement(statement) => {
                        function.children.push(convert_ast_statement(statement))
                    }
                }
            }

            function
        }
    }
}

fn convert_ast_declaration(ast_declaration: &AstDeclaration) -> Node {
    match ast_declaration {
        AstDeclaration::Declaration(identifier, expression) => {
            match expression {
                Some(expr) => {
                    let mut declaration = Node::new(format!("{}=exp", identifier));
                    declaration.children.push(convert_ast_expression(expr));

                    declaration
                },
                None => {
                    let declaration = Node::new(format!("{}", identifier));
                    declaration
                }
            }
        }
    }
}

fn convert_ast_statement(ast_statement: &AstStatement) -> Node {
    match ast_statement {
        AstStatement::Return(expr) => {
            let mut statement = Node::new("Return(exp)".to_string());
            statement.children.push(convert_ast_expression(expr));

            statement
        },
        AstStatement::Expression(expr) => {
            let mut expression = Node::new("ExprStatement(exp)".to_string());
            expression.children.push(convert_ast_expression(expr));

            expression
        },
        AstStatement::Null => {
            Node::new("Null".to_string())
        }
    }
}

fn convert_ast_expression(ast_expression: &AstExpression) -> Node {
    match ast_expression {
        AstExpression::Constant(num) => {
            Node::new(format!("Constant({})", num))
        }
        AstExpression::Binary(operator, left, right) => {
            let mut binary_node = Node::new("Binary(operator, left, right)".to_string());
            binary_node.children.push(convert_ast_expression(left));

            let operator_node = match operator {
                AstBinaryOp::Add => Node::new("Add".into()),
                AstBinaryOp::And => Node::new("And".into()),
                AstBinaryOp::Divide => Node::new("Divide".into()),
                AstBinaryOp::Equal => Node::new("Equal".into()),
                AstBinaryOp::GreaterOrEqual => Node::new("GreaterOrEqual".into()),
                AstBinaryOp::GreaterThan => Node::new("GreaterThan".into()),
                AstBinaryOp::LessOrEqual => Node::new("LessOrEqual".into()),
                AstBinaryOp::LessThan => Node::new("LessThan".into()),
                AstBinaryOp::Multiply => Node::new("Multiply".into()),
                AstBinaryOp::NotEqual => Node::new("NotEqual".into()),
                AstBinaryOp::Or => Node::new("Or".into()),
                AstBinaryOp::Remainder => Node::new("Remainder".into()),
                AstBinaryOp::Subtract => Node::new("Subtract".into())
            };

            binary_node.children.push(operator_node);
            binary_node.children.push(convert_ast_expression(right));

            binary_node
        }
        AstExpression::Unary(operator, expr) => {
            let mut unary_node = Node::new("Unary(operator, expr)".into());

            let operator_node = match operator {
                AstUnaryOp::Not => Node::new("Not".into()),
                AstUnaryOp::Complement => Node::new("Complement".into()),
                AstUnaryOp::Negate => Node::new("Negate".into()),
            };

            unary_node.children.push(operator_node);
            unary_node.children.push(convert_ast_expression(expr));

            unary_node
        },
        AstExpression::Var(identifier) => {
            Node::new(format!("Var({})", identifier))
        },
        AstExpression::Assignment(identifier, expression) => {
            let mut assignment = Node::new(format!("Assignment(ident, expr)"));
            assignment.children.push(convert_ast_expression(&*identifier));
            assignment.children.push(convert_ast_expression(&*expression));

            assignment
        }
    }
}