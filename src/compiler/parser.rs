use crate::compiler::token::{Token};
use crate::storage::ast::{Program, Function, Statement, Expression};

#[derive(Debug, PartialEq)]
pub struct ParserErr(String);

pub fn parse_program(tokens: &mut Vec<Token>) -> Result<Program, ParserErr> {
    let function = match parse_function(tokens) {
        Ok(exp) => exp,
        Err(err) => return Err(err),
    };

    if tokens.len() != 0 {
        return Err(ParserErr("Syntax error!".to_string()));
    }
    else {
        Ok(Program::ProgramNode(function))
    }
}

fn parse_function(tokens: &mut Vec<Token>) -> Result<Function, ParserErr> {
    match expect(&Token::Integer, tokens) {
        Err(err) => return Err(err),
        _ => tokens.remove(0)
    };

    let identifier = if let Some(Token::Identifier(identifier_name)) = tokens.first() {
        identifier_name.clone()
    }
    else {
        return Err(ParserErr(format!("expected {:?}, got {:?}", &Token::Identifier(String::new()), tokens.first().unwrap())))
    };

    tokens.remove(0);


    match expect(&Token::OpenParen, tokens) {
        Err(err) => return Err(err),
        _ => tokens.remove(0)
    };

    match expect(&Token::Void, tokens) {
        Err(err) => return Err(err),
        _ => tokens.remove(0)
    };

    match expect(&Token::CloseParen, tokens) {
        Err(err) => return Err(err),
        _ => tokens.remove(0)
    };

    match expect(&Token::OpenBrace, tokens) {
        Err(err) => return Err(err),
        _ => tokens.remove(0)
    };

    let statement = match parse_statement(tokens) {
        Ok(exp) => exp,
        Err(err) => return Err(err),
    };

    match expect(&Token::CloseBrace, tokens) {
        Err(err) => return Err(err),
        _ => tokens.remove(0)
    };

    Ok(Function::FunctionNode(Token::Identifier(identifier), statement))
}

fn parse_statement(tokens: &mut Vec<Token>) -> Result<Statement, ParserErr> {
    match expect(&Token::Return, tokens) {
        Ok(()) => tokens.remove(0),
        Err(err) => return Err(err),
    };

    let return_val = match parse_expression(tokens) {
        Ok(exp) => {
            tokens.remove(0);
            exp
        },
        Err(err) => return Err(err),
    };

    match expect(&Token::Semicolon, tokens) {
        Err(err) => return Err(err),
        _ => tokens.remove(0)
    };

    Ok(Statement::ReturnNode(return_val))
}

fn parse_expression(tokens: &mut Vec<Token>) -> Result<Expression, ParserErr> {
    if let Some(Token::Constant(some)) = tokens.first() {
        Ok(Expression::ConstantNode(Token::Constant(some.clone())))
    }
    else {
        Err(ParserErr(format!("expected {:?}, got {:?}", &Token::Constant(0), tokens.first().unwrap())))
    }
}

pub fn expect(expected: &Token, tokens: &Vec<Token>) -> Result<(), ParserErr>  {
    let actual = tokens.first().unwrap();

    match (expected, actual) {
        (Token::Identifier(_), Token::Identifier(_)) => Ok(()),
        (Token::Constant(_), Token::Constant(_)) => Ok(()),
        _ => {
            match expected == actual
            {
                true => Ok(()),
                false => Err(ParserErr(format!("expected {:?}, got {:?}", expected, actual))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::token::Token;
    use crate::compiler::parser::{expect, parse_expression, parse_function, parse_program, parse_statement, ParserErr};
    use crate::storage::ast::{Expression, Function, Program, Statement};

    #[test]
    fn expect_basic_pass() {
        let tokens = vec![Token::Constant(15)];

        let expected = &Token::Constant(0);
        assert_eq!(expect(expected, &tokens), Ok(()));
    }

    #[test]
    fn expect_basic_err() {
        let tokens = vec![Token::Constant(15)];

        let expected = &Token::Identifier("main".to_string());
        assert_eq!(expect(expected, &tokens),
                   Err(ParserErr(format!("expected {:?}, got {:?}", expected, tokens.first().unwrap()))));
    }

    #[test]
    fn parse_expression_basic_pass() {
        let mut tokens = vec![Token::Constant(15), Token::Semicolon];

        let expr = parse_expression(&mut tokens);

        assert_eq!(expr, Ok(Expression::ConstantNode(Token::Constant(15))));

        let mut tokens = vec![Token::Constant(15)];

        let expr = parse_expression(&mut tokens);

        assert_eq!(expr, Ok(Expression::ConstantNode(Token::Constant(15))));
    }

    #[test]
    fn parse_expression_basic_fail() {
        let mut tokens = vec![Token::Semicolon];

        let expr = parse_expression(&mut tokens);

        assert_eq!(expr, Err(ParserErr(format!("expected {:?}, got {:?}", &Token::Constant(0), tokens.first().unwrap()))));
    }

    #[test]
    fn parse_statement_pass_with_left_tokens() {
        let mut tokens = vec![Token::Return, Token::Constant(2), Token::Semicolon, Token::CloseBrace];

        let statement = parse_statement(&mut tokens);

        assert_eq!(statement, Ok(Statement::ReturnNode(Expression::ConstantNode(Token::Constant(2)))));
        assert_eq!(tokens, vec![Token::CloseBrace]);
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn parse_statement_pass_with_no_more_tokens() {
        let mut tokens = vec![Token::Return, Token::Constant(2), Token::Semicolon];

        let statement = parse_statement(&mut tokens);

        assert_eq!(statement, Ok(Statement::ReturnNode(Expression::ConstantNode(Token::Constant(2)))));
        assert_eq!(tokens.len(), 0);
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn parse_statement_fail_with_incorrect_token_at_beginning() {
        let mut tokens = vec![Token::Integer, Token::Constant(2), Token::Semicolon];

        let statement = parse_statement(&mut tokens);

        assert_eq!(statement, Err(ParserErr(format!("expected {:?}, got {:?}", &Token::Return, tokens.first().unwrap()))));
        assert_eq!(tokens, vec![Token::Integer, Token::Constant(2), Token::Semicolon]);
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn parse_statement_fail_with_incorrect_token() {
        let mut tokens = vec![Token::Return, Token::Identifier("main".to_string()), Token::Semicolon];

        let statement = parse_statement(&mut tokens);

        assert_eq!(statement, Err(ParserErr(format!("expected {:?}, got {:?}", &Token::Constant(0), tokens.first().unwrap()))));
        assert_eq!(tokens, vec![Token::Identifier("main".to_string()), Token::Semicolon]);
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn parse_function_pass() {
        let mut tokens = vec![
            Token::Integer,
            Token::Identifier("main".to_string()),
            Token::OpenParen,
            Token::Void,
            Token::CloseParen,
            Token::OpenBrace,
            Token::Return,
            Token::Constant(2),
            Token::Semicolon,
            Token::CloseBrace];

        let statement = parse_function(&mut tokens);

        assert_eq!(statement, Ok(Function::FunctionNode(Token::Identifier("main".to_string()), Statement::ReturnNode(Expression::ConstantNode(Token::Constant(2))))));
        assert_eq!(tokens, vec![]);
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn parse_function_fail() {
        let mut tokens = vec![
            Token::Integer,
            Token::Identifier("main".to_string()),
            Token::Void,
            Token::CloseParen,
            Token::OpenBrace,
            Token::Return,
            Token::Constant(2),
            Token::Semicolon,
            Token::CloseBrace];

        let statement = parse_function(&mut tokens);

        assert_eq!(statement, Err(ParserErr(format!("expected {:?}, got {:?}", &Token::OpenParen, tokens.first().unwrap()))));
        assert_eq!(tokens, vec![
            Token::Void,
            Token::CloseParen,
            Token::OpenBrace,
            Token::Return,
            Token::Constant(2),
            Token::Semicolon,
            Token::CloseBrace
        ]);
        assert_eq!(tokens.len(), 7);
    }

    #[test]
    fn parse_program_pass() {
        let mut tokens = vec![
            Token::Integer,
            Token::Identifier("main".to_string()),
            Token::OpenParen,
            Token::Void,
            Token::CloseParen,
            Token::OpenBrace,
            Token::Return,
            Token::Constant(2),
            Token::Semicolon,
            Token::CloseBrace];

        let statement = parse_program(&mut tokens);

        assert_eq!(statement, Ok(Program::ProgramNode(Function::FunctionNode(Token::Identifier("main".to_string()), Statement::ReturnNode(Expression::ConstantNode(Token::Constant(2)))))));
    }

    #[test]
    fn parse_program_fail_too_many_tokens() {
        let mut tokens = vec![
            Token::Integer,
            Token::Identifier("main".to_string()),
            Token::OpenParen,
            Token::Void,
            Token::CloseParen,
            Token::OpenBrace,
            Token::Return,
            Token::Constant(2),
            Token::Semicolon,
            Token::CloseBrace,
            Token::CloseBrace];

        let statement = parse_program(&mut tokens);

        assert_eq!(statement, Err(ParserErr("Syntax error!".to_string())));
    }
}