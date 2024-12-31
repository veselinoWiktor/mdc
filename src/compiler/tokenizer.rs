//! C tokenizer that produces ['Token'] instances.

use regex::{Regex};

use super::token::{Token};

struct TokenDef {
    regex: Regex,
    converter: Box<dyn Fn(&str) -> Token>,
}

#[derive(Debug, PartialEq)]
pub struct TokenizeError(String);

impl TokenDef {
    fn new(pattern: &str, converter: Box<dyn Fn(&str) -> Token>) -> Self {
        TokenDef {
            regex: Regex::new(&format!("^{}", pattern)).unwrap(),
            converter,
        }
    }
}

fn convert_identifier(s: &str) -> Token {
    match s {
        "int" => Token::Integer,
        "return" => Token::Return,
        "void" => Token::Void,
        _ => Token::Identifier(s.to_string()),
    }
}

fn convert_int(s: &str) -> Token {
    Token::Constant(s.parse().unwrap())
}

/// Define token patterns and converters
fn token_definitions() -> Vec<TokenDef> {
    vec![
        TokenDef::new(r"[A-Za-z_][A-Za-z0-9_]*\b", Box::new(convert_identifier)),
        TokenDef::new(r"[0-9]+\b", Box::new(convert_int)),
        TokenDef::new(r"\(", Box::new(|_| Token::OpenParen)),
        TokenDef::new(r"\)", Box::new(|_| Token::CloseParen)),
        TokenDef::new(r"\{", Box::new(|_| Token::OpenBrace)),
        TokenDef::new(r"\}", Box::new(|_| Token::CloseBrace)),
        TokenDef::new(r";", Box::new(|_| Token::Semicolon)),
    ]
}

/// Count leading whitespace
fn count_leading_whitespace(s: &str) -> Option<usize> {
    // ^\s+ matches 1 or more [\n, ' ', \t, ..] at the beginning of the str
    let re = Regex::new(r"^\s+").unwrap();

    // matches all whitespaces and then maps them to the end of the match
    // which returns how long the whitespace characters are
    re.find(s).map(|m| m.end())
}

// Try to find a matching token at the start of the input
fn find_match(input: &str, token_def: &TokenDef) -> Option<(String, Token)> {
    if let Some(mat) = token_def.regex.find(input) {
        let matched_str = mat.as_str();
        let token = (token_def.converter)(matched_str);
        Some((matched_str.to_string(), token))
    } else {
        None
    }
}

// Main lexing function
pub fn tokenize(mut input: &str) -> Result<Vec<Token>, TokenizeError> {
    let token_defs = token_definitions();
    let mut tokens = Vec::new();

    while !input.is_empty() {
        // skip whitespace
        if let Some(ws_len) = count_leading_whitespace(input) {
            input = &input[ws_len..];
        }

        // if file ends with whitespaces we should break here
        if input.is_empty() {
            break;
        }

        // try to match tokens
        let matches: Vec<_> = token_defs
            .iter()
            .filter_map(|td| {
                find_match(input, td)
            })
            .collect();

        // error if no matches
        if matches.is_empty() {
            return Err(TokenizeError("Unable to find match".to_string()));
        }

        // get the longest match
        let (longest_match, token) = matches
            .into_iter()
            .max_by_key(|(matched_str, _)| matched_str.len())
            .unwrap();

        tokens.push(token);
        input = &input[longest_match.len()..];
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use crate::compiler::token::{Token};
    use crate::compiler::tokenizer::{tokenize, TokenizeError};

    #[test]
    fn basic() {
        let code =
        "int main(void) {
            return 2;
        }";

        assert_eq!(tokenize(code).unwrap(), vec![
            Token::Integer,
            Token::Identifier("main".to_string()),
            Token::OpenParen,
            Token::Void,
            Token::CloseParen,
            Token::OpenBrace,
            Token::Return,
            Token::Constant(2),
            Token::Semicolon,
            Token::CloseBrace]);
    }

    #[test]
    fn basic_fail() {
        let code =
        "int main(void) {
            int a = 5;
            return a;
        }";

        assert_eq!(tokenize(code).unwrap_err(), TokenizeError("Unable to find match".to_string()));
    }
}
