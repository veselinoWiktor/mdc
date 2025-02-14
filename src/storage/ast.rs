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

#[derive(Debug, PartialEq)]
pub enum AstProgram {
    Program(AstFunctionDefinition)
}

#[derive(Debug, PartialEq)]
pub enum AstFunctionDefinition {
    Function(String, Vec<AstBlockItem>)
}

#[derive(Debug, PartialEq)]
pub enum AstBlockItem {
    Statement(AstStatement),
    Declaration(AstDeclaration)
}

#[derive(Debug, PartialEq)]
pub enum AstDeclaration {
    Declaration(String, Option<AstExpression>)
}

#[derive(Debug, PartialEq)]
pub enum AstStatement {
    Return(AstExpression),
    Expression(AstExpression),
    Null
}

#[derive(Debug, PartialEq)]
pub enum AstExpression {
    Constant(i32),
    Var(String), // Var(identifier)
    Unary(AstUnaryOp, Box<AstExpression>),
    Binary(AstBinaryOp, Box<AstExpression>, Box<AstExpression>),
    Assignment(Box<AstExpression>, Box<AstExpression>),
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