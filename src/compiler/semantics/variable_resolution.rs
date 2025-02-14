use std::collections::HashMap;
use crate::storage::ast::{AstBlockItem, AstDeclaration, AstExpression, AstFunctionDefinition, AstProgram, AstStatement};

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError {
    UndeclaredVariable,
    DuplicateVariable,
    InvalidLValue,
}

pub fn resolve_program(ast_program: AstProgram) -> Result<AstProgram, SemanticError> {
    match ast_program {
        AstProgram::Program(function_definitions) => {
            Ok(AstProgram::Program(resolve_function(function_definitions)?))
        }
    }
}

fn resolve_function(ast_function_def: AstFunctionDefinition) -> Result<AstFunctionDefinition, SemanticError> {
    match ast_function_def {
        AstFunctionDefinition::Function(identifier, body) => {
            let mut variable_map: HashMap<String, String> = HashMap::new();

            let mut new_body = vec![];

            for block_item in body {
                match block_item {
                    AstBlockItem::Declaration(declaration) => {
                        new_body.push(AstBlockItem::Declaration(resolve_declaration(declaration, &mut variable_map)?))
                    },
                    AstBlockItem::Statement(statement) => {
                        new_body.push(AstBlockItem::Statement(resolve_statement(statement, &variable_map)?))
                    }
                }
            };

            Ok(AstFunctionDefinition::Function(identifier, new_body))
        }
    }
}

fn resolve_statement(ast_statement: AstStatement, variable_map: &HashMap<String, String>) -> Result<AstStatement, SemanticError> {
    match ast_statement {
        AstStatement::Return(expr) => Ok(AstStatement::Return(resolve_expression(expr, &variable_map)?)),
        AstStatement::Expression(expr) => Ok(AstStatement::Expression(resolve_expression(expr, &variable_map)?)),
        AstStatement::Null => Ok(AstStatement::Null),
    }
}

fn resolve_declaration(ast_declaration: AstDeclaration, variable_map: &mut HashMap<String, String>) -> Result<AstDeclaration, SemanticError> {
    match ast_declaration {
        AstDeclaration::Declaration(name, init) => {
            if variable_map.contains_key(&name) {
                return Err(SemanticError::DuplicateVariable)
            }

            let unique_name = format!("{}.0", name.clone());

            variable_map.insert(name.clone(), unique_name.clone());


            if let Some(expr) = init {
                Ok(AstDeclaration::Declaration(unique_name, Some(resolve_expression(expr, &variable_map)?)))
            } else {
                Ok(AstDeclaration::Declaration(unique_name, init))
            }
        }
    }

}

fn resolve_expression(ast_expression: AstExpression, variable_map: &HashMap<String, String>) -> Result<AstExpression, SemanticError> {
    match ast_expression {
        AstExpression::Assignment(left, right) => {
            if let AstExpression::Var(_) = *left {
                Ok(AstExpression::Assignment(Box::new(resolve_expression(*left, variable_map)?), Box::new(resolve_expression(*right, variable_map)?)))
            } else {
                Err(SemanticError::InvalidLValue)
            }
        }
        AstExpression::Var(identifier) => {
            if variable_map.contains_key(&identifier) {
                Ok(AstExpression::Var(variable_map.get(&identifier).unwrap().clone()))
            } else {
                Err(SemanticError::InvalidLValue)
            }
        },
        AstExpression::Binary(bin_op, left, right) => {
            Ok(AstExpression::Binary(bin_op, Box::new(resolve_expression(*left, variable_map)?), Box::new(resolve_expression(*right, variable_map)?)))
        }
        AstExpression::Unary(un_op, expr) => {
            Ok(AstExpression::Unary(un_op, Box::new(resolve_expression(*expr, variable_map)?)))
        },
        _ => Ok(ast_expression)
    }
}