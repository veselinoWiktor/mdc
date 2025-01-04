//! Three-address code(TAC)-KY makes it fancier.
//!
//! TACKY ASDL definition:
//! ```
//! <program> ::= <function>
//! <function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
//! <statement> ::= "return" <exp> ";"
//! <exp> ::= <factor> | <exp> <binop> <exp>
//! <factor> ::= <int> | <unop> <factor> | "(" <exp> ")"
//! <unop> ::= "-" | "~"
//! <binop> ::= "-" | "+" | "*" | "/" | "%"
//! <identifier> ::= ? An identifier token ?
//! <int> ::= ? A constant token ?
//! ```

use std::fmt::{Display, Formatter};

pub trait PrettyFormatter {
    fn pretty_format(&self, indent: usize) -> String;
}

#[derive(Debug, PartialEq)]
pub enum AstProgram {
    Program(AstFunctionDefinition)
}

impl PrettyFormatter for AstProgram {
    fn pretty_format(&self, indent: usize) -> String {
        let mut result = String::new();
        result.push_str(&" ".repeat(indent));
        result.push_str("Program(\n");
        match self {
            AstProgram::Program(function) => {
                result.push_str(function.pretty_format(indent + 4).as_str());
                result.push_str("\n");
            }
        }
        result.push_str(&" ".repeat(indent));
        result.push_str(")");
        result
    }
}

impl Display for AstProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstProgram::Program(function) => write!(f, "Program(\n{}\n)", function.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AstFunctionDefinition {
    Function(String, AstStatement)
}

impl PrettyFormatter for AstFunctionDefinition {
    fn pretty_format(&self, indent: usize) -> String {
        let mut result = String::new();
        result.push_str(&" ".repeat(indent));
        result.push_str("Function(\n");
        match self {
            AstFunctionDefinition::Function(identifier, statement) => {
                result.push_str(&" ".repeat(indent + 4));
                result.push_str(format!("name=\"{}\"\n", identifier).as_str());
                result.push_str(&" ".repeat(indent + 4));
                result.push_str("body="); // currently we know body has only one instruction
                result.push_str(statement.pretty_format(indent + 4).as_str());
                result.push_str("\n");
            }
        }
        result.push_str(&" ".repeat(indent));
        result.push_str(")");
        result
    }
}

impl Display for AstFunctionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AstFunctionDefinition::Function(identifier, statement) => {
                write!(f, "\tFunction(\n\t\tname=\"{}\",\n\t\tbody={}\n\t)", identifier.clone(), statement.to_string())
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AstStatement {
    Return(AstExpression)
}

impl PrettyFormatter for AstStatement {
    fn pretty_format(&self, indent: usize) -> String {
        let mut result = String::new();
        match self {
            AstStatement::Return(expression) => {
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

impl Display for AstStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AstStatement::Return(expression) => {
                write!(f, "Return(\n{}\n\t\t)", expression.to_string())
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AstExpression {
    Constant(i32),
    Unary(AstUnaryOp, Box<AstExpression>),
    Binary(AstBinaryOp, Box<AstExpression>, Box<AstExpression>)
}

impl PrettyFormatter for AstExpression {
    fn pretty_format(&self, indent: usize) -> String {
        let mut result = String::new();
        result.push_str(&" ".repeat(indent));
        match self {
            AstExpression::Constant(num) => {
                result.push_str(format!("Constant({})", num).as_str());
            }
            _ => unreachable!()
        }
        result.push_str(&" ".repeat(indent));
        result
    }
}

impl Display for AstExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AstExpression::Constant(num) => {
                write!(f, "\t\t\tConstant({})", num.to_string())
            },
            _ => unreachable!()
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AstUnaryOp{
    Complement,
    Negate,
    Not
}

#[derive(Debug, PartialEq)]
pub enum AstBinaryOp{
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}