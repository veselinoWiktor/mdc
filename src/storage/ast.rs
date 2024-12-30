use crate::compiler::token::Token;

enum Program {
    ProgramNode(Function)
}

enum Function {
    FunctionNode(Token::Identifier, Statement)
}

enum Statement {
    ReturnNode(Expression)
}

enum Expression {
    ConstantNode(Token::Constant)
}

