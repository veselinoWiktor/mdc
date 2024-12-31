use std::fmt::{Display, Formatter};
use crate::compiler::token::Token;

pub trait PrettyFormatter {
    fn pretty_format(&self, indent: usize) -> String;
}

#[derive(Debug, PartialEq)]
pub enum Program {
    ProgramNode(Function)
}

impl PrettyFormatter for Program {
    fn pretty_format(&self, indent: usize) -> String {
        let mut result = String::new();
        result.push_str(&" ".repeat(indent));
        result.push_str("Program(\n");
        match self {
            Program::ProgramNode(function) => {
                result.push_str(function.pretty_format(indent + 4).as_str());
                result.push_str("\n");
            }
        }
        result.push_str(&" ".repeat(indent));
        result.push_str(")");
        result
    }
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

impl PrettyFormatter for Function {
    fn pretty_format(&self, indent: usize) -> String {
        let mut result = String::new();
        result.push_str(&" ".repeat(indent));
        result.push_str("Function(\n");
        match self {
            Function::FunctionNode(Token::Identifier(identifier), statement) => {
                result.push_str(&" ".repeat(indent + 4));
                result.push_str(format!("name=\"{}\"\n", identifier).as_str());
                result.push_str(&" ".repeat(indent + 4));
                result.push_str("body="); // currently we know body has only one instruction
                result.push_str(statement.pretty_format(indent + 4).as_str());
                result.push_str("\n");
            }
            _ => unreachable!()
        }
        result.push_str(&" ".repeat(indent));
        result.push_str(")");
        result
    }
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

impl PrettyFormatter for Statement {
    fn pretty_format(&self, indent: usize) -> String {
        let mut result = String::new();
        match self {
            Statement::ReturnNode(expression) => {
                result.push_str("Return(\n");
                result.push_str(expression.pretty_format(indent + 4).as_str());
                result.push_str("\n");
            }
        }
        result.push_str(&" ".repeat(indent));
        result.push_str(")");
        result
    }
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

impl PrettyFormatter for Expression {
    fn pretty_format(&self, indent: usize) -> String {
        let mut result = String::new();
        result.push_str(&" ".repeat(indent));
        match self {
            Expression::ConstantNode(Token::Constant(num)) => {
                result.push_str(format!("Constant({})", num).as_str());
            }
            _ => unreachable!()
        }
        result.push_str(&" ".repeat(indent));
        result
    }
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

