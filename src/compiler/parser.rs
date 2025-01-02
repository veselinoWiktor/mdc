use crate::compiler::token::Token;
use crate::storage::ast::{
    AstExpression, AstFunctionDefinition, AstProgram, AstStatement, AstUnaryOp,
};

#[derive(Debug, PartialEq)]
pub struct ParserErr(String);

pub fn parse_program(tokens: &mut Vec<Token>) -> Result<AstProgram, ParserErr> {
    let function = match parse_function(tokens) {
        Ok(exp) => exp,
        Err(err) => return Err(err),
    };

    if tokens.len() != 0 {
        return Err(ParserErr("Syntax error!".to_string()));
    } else {
        Ok(AstProgram::Program(function))
    }
}

fn parse_function(tokens: &mut Vec<Token>) -> Result<AstFunctionDefinition, ParserErr> {
    expect(&Token::Integer, tokens)?;
    tokens.remove(0);

    let identifier = if let Some(Token::Identifier(identifier_name)) = tokens.first() {
        identifier_name.clone()
    } else {
        return Err(ParserErr(format!(
            "expected {:?}, got {:?}",
            &Token::Identifier(String::new()),
            tokens.first().unwrap()
        )));
    };
    tokens.remove(0);

    expect_sequence_with_remove(
        &vec![
            Token::OpenParen,
            Token::Void,
            Token::CloseParen,
            Token::OpenBrace,
        ],
        tokens,
    )?;

    let statement = match parse_statement(tokens) {
        Ok(exp) => exp,
        Err(err) => return Err(err),
    };

    expect(&Token::CloseBrace, tokens)?;
    tokens.remove(0);

    Ok(AstFunctionDefinition::Function(
        Token::Identifier(identifier),
        statement,
    ))
}

fn parse_statement(tokens: &mut Vec<Token>) -> Result<AstStatement, ParserErr> {
    match expect(&Token::Return, tokens) {
        Ok(()) => tokens.remove(0),
        Err(err) => return Err(err),
    };

    let return_val = match parse_expression(tokens) {
        Ok(exp) => {
            tokens.remove(0);
            exp
        }
        Err(err) => return Err(err),
    };

    match expect(&Token::Semicolon, tokens) {
        Err(err) => return Err(err),
        _ => tokens.remove(0),
    };

    Ok(AstStatement::Return(return_val))
}

/// ```<exp> ::= <int> | <unop> <exp> | "(" <exp> ")"```
fn parse_expression(tokens: &mut Vec<Token>) -> Result<AstExpression, ParserErr> {
    match tokens.first() {
        Some(token) => match token {
            Token::Constant(num) => Ok(AstExpression::Constant(Token::Constant(num.clone()))),
            Token::Tilde | Token::Hyphen => {
                let operator = parse_unary_operator(tokens)?;
                tokens.remove(0);
                let inner_expr = parse_expression(tokens)?;
                Ok(AstExpression::Unary(operator, Box::new(inner_expr)))
            }
            Token::OpenParen => {
                tokens.remove(0);
                let inner_expr = parse_expression(tokens)?;
                tokens.remove(0);
                expect(&Token::CloseParen, tokens)?;

                Ok(inner_expr)
            }
            _ => Err(ParserErr(format!(
                "expected {:?}, got {:?}",
                &Token::Constant(0),
                tokens.first().unwrap()
            ))),
        },
        None => Err(ParserErr("No more tokens".to_string())),
    }
}

fn parse_unary_operator(tokens: &mut Vec<Token>) -> Result<AstUnaryOp, ParserErr> {
    match tokens.first() {
        Some(token) => match token {
            Token::Tilde => Ok(AstUnaryOp::Complement),
            Token::Hyphen => Ok(AstUnaryOp::Negate),
            _ => Err(ParserErr(format!(
                "expected token signifying unary operation, got {:?}",
                tokens.first().unwrap()
            ))),
        },
        None => Err(ParserErr("No more tokens".to_string())),
    }
}

pub fn expect(expected: &Token, tokens: &Vec<Token>) -> Result<(), ParserErr> {
    let actual = tokens.first().unwrap();

    match (expected, actual) {
        (Token::Identifier(_), Token::Identifier(_)) => Ok(()),
        (Token::Constant(_), Token::Constant(_)) => Ok(()),
        _ => match expected == actual {
            true => Ok(()),
            false => Err(ParserErr(format!(
                "expected {:?}, got {:?}",
                expected, actual
            ))),
        },
    }
}

// pub fn expect_sequence(expected: &Vec<Token>, tokens: &Vec<Token>) -> Result<(), ParserErr> {
//     for token in expected {
//         expect(token, tokens)?;
//     }
//     Ok(())
// }

pub fn expect_sequence_with_remove(
    expected: &Vec<Token>,
    tokens: &mut Vec<Token>,
) -> Result<(), ParserErr> {
    for token in expected {
        expect(token, tokens)?;
        tokens.remove(0);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::compiler::parser::{
        expect, parse_expression, parse_function, parse_program, parse_statement, ParserErr,
    };
    use crate::compiler::token::Token;
    use crate::storage::ast::{AstExpression, AstFunctionDefinition, AstProgram, AstStatement};

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
        assert_eq!(
            expect(expected, &tokens),
            Err(ParserErr(format!(
                "expected {:?}, got {:?}",
                expected,
                tokens.first().unwrap()
            )))
        );
    }

    #[test]
    fn parse_expression_basic_pass() {
        let mut tokens = vec![Token::Constant(15), Token::Semicolon];

        let expr = parse_expression(&mut tokens);

        assert_eq!(expr, Ok(AstExpression::Constant(Token::Constant(15))));

        let mut tokens = vec![Token::Constant(15)];

        let expr = parse_expression(&mut tokens);

        assert_eq!(expr, Ok(AstExpression::Constant(Token::Constant(15))));
    }

    #[test]
    fn parse_expression_basic_fail() {
        let mut tokens = vec![Token::Semicolon];

        let expr = parse_expression(&mut tokens);

        assert_eq!(
            expr,
            Err(ParserErr(format!(
                "expected {:?}, got {:?}",
                &Token::Constant(0),
                tokens.first().unwrap()
            )))
        );
    }

    #[test]
    fn parse_statement_pass_with_left_tokens() {
        let mut tokens = vec![
            Token::Return,
            Token::Constant(2),
            Token::Semicolon,
            Token::CloseBrace,
        ];

        let statement = parse_statement(&mut tokens);

        assert_eq!(
            statement,
            Ok(AstStatement::Return(AstExpression::Constant(
                Token::Constant(2)
            )))
        );
        assert_eq!(tokens, vec![Token::CloseBrace]);
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn parse_statement_pass_with_no_more_tokens() {
        let mut tokens = vec![Token::Return, Token::Constant(2), Token::Semicolon];

        let statement = parse_statement(&mut tokens);

        assert_eq!(
            statement,
            Ok(AstStatement::Return(AstExpression::Constant(
                Token::Constant(2)
            )))
        );
        assert_eq!(tokens.len(), 0);
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn parse_statement_fail_with_incorrect_token_at_beginning() {
        let mut tokens = vec![Token::Integer, Token::Constant(2), Token::Semicolon];

        let statement = parse_statement(&mut tokens);

        assert_eq!(
            statement,
            Err(ParserErr(format!(
                "expected {:?}, got {:?}",
                &Token::Return,
                tokens.first().unwrap()
            )))
        );
        assert_eq!(
            tokens,
            vec![Token::Integer, Token::Constant(2), Token::Semicolon]
        );
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn parse_statement_fail_with_incorrect_token() {
        let mut tokens = vec![
            Token::Return,
            Token::Identifier("main".to_string()),
            Token::Semicolon,
        ];

        let statement = parse_statement(&mut tokens);

        assert_eq!(
            statement,
            Err(ParserErr(format!(
                "expected {:?}, got {:?}",
                &Token::Constant(0),
                tokens.first().unwrap()
            )))
        );
        assert_eq!(
            tokens,
            vec![Token::Identifier("main".to_string()), Token::Semicolon]
        );
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
            Token::CloseBrace,
        ];

        let statement = parse_function(&mut tokens);

        assert_eq!(
            statement,
            Ok(AstFunctionDefinition::Function(
                Token::Identifier("main".to_string()),
                AstStatement::Return(AstExpression::Constant(Token::Constant(2)))
            ))
        );
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
            Token::CloseBrace,
        ];

        let statement = parse_function(&mut tokens);

        assert_eq!(
            statement,
            Err(ParserErr(format!(
                "expected {:?}, got {:?}",
                &Token::OpenParen,
                tokens.first().unwrap()
            )))
        );
        assert_eq!(
            tokens,
            vec![
                Token::Void,
                Token::CloseParen,
                Token::OpenBrace,
                Token::Return,
                Token::Constant(2),
                Token::Semicolon,
                Token::CloseBrace
            ]
        );
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
            Token::CloseBrace,
        ];

        let statement = parse_program(&mut tokens);

        assert_eq!(
            statement,
            Ok(AstProgram::Program(
                AstFunctionDefinition::Function(
                    Token::Identifier("main".to_string()),
                    AstStatement::Return(AstExpression::Constant(Token::Constant(2)))
                )
            ))
        );
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
            Token::CloseBrace,
        ];

        let statement = parse_program(&mut tokens);

        assert_eq!(statement, Err(ParserErr("Syntax error!".to_string())));
    }
}
