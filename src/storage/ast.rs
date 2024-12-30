use std::convert::identity;
use std::fmt::{Display, Formatter};
use crate::compiler::token::Token;

#[derive(Debug, PartialEq)]
pub enum Program {
    ProgramNode(Function)
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Program::ProgramNode(function) => write!(f, "Program(\n{}\n)", function.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Function {
    FunctionNode(Token, Statement)
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::FunctionNode(Token::Identifier(identifier), statement) => {
                write!(f, "\tFunction(\n\t\tname=\"{}\",\n\t\tbody={}\n\t)", identifier.clone(), statement.to_string())
            },
            _ => unreachable!()
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    ReturnNode(Expression)
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::ReturnNode(expression) => {
                write!(f, "Return(\n{}\n\t\t)", expression.to_string())
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    ConstantNode(Token)
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::ConstantNode(Token::Constant(num)) => {
                write!(f, "\t\t\tConstant({})", num.to_string())
            },
            _ => unreachable!()
        }
    }
}

