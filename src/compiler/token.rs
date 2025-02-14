//! C token definitions.

/// C tokens.
#[derive(Debug, PartialEq)]
pub enum Token {
    // Tokens with contents
    Identifier(String),
    Constant(i32),
    // Keywords
    Integer,
    Void,
    Return,
    // Punctuation
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Hyphen,
    DoubleHyphen,
    Tilde,
    Plus,
    DoublePlus,
    Asterisk,
    ForwardSlash,
    Percent,
    Equal,
    // Logical tokens
    LogicalNot,
    LogicalAnd,
    LogicalOr,
    LogicalEqual,
    LogicalNotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
}