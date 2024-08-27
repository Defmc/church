use std::{iter::Peekable, ops::Range};

use crate::{UBody, UTerm};
use logos::Logos;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum ParserBodyError {
    #[error("Missing a dot (`.`) after a variable in a lambda abstraction")]
    MissingDot,

    #[error("Unexpected token {0:?}")]
    UnexpectedToken(std::result::Result<Token, ()>),

    #[error("Unexpected eof")]
    UnexpectedEof,

    #[error("Expected Ident wasn't found")]
    IdentNotFound,

    #[error("Missing a closing parenthesis")]
    ParenUnclosed,
}
#[derive(Logos, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("λ")]
    #[token("\\")]
    Lambda,

    #[token(".")]
    #[token("->")]
    Dot,

    #[token("(")]
    OpenParen,

    #[token(")")]
    CloseParen,

    #[regex("[a-zA-Z0-9α-κμ-ωΑ-ΚΜ-Ω_]+")]
    Ident,
}

pub type LexerTy = (std::result::Result<Token, ()>, Range<usize>);

pub type Result<T> = std::result::Result<T, ParserBodyError>;

pub fn try_from_str(s: &str) -> Result<UTerm> {
    try_from_iter(&mut Token::lexer(s).spanned().peekable(), s)
}

pub fn try_from_iter<I>(it: &mut Peekable<I>, src: &str) -> Result<UTerm>
where
    I: Iterator<Item = LexerTy>,
{
    let mut result = try_atom(it, src)?;
    while let Some((t, _)) = it.peek() {
        match t {
            Ok(Token::CloseParen) => break,
            _ => result = UTerm::from(UBody::App(result, try_atom(it, src)?)),
        }
    }
    Ok(result)
}

pub fn try_atom<I>(it: &mut Peekable<I>, src: &str) -> Result<UTerm>
where
    I: Iterator<Item = LexerTy>,
{
    let (tok, span) = it.next().ok_or(ParserBodyError::UnexpectedEof)?;
    match tok.as_ref().unwrap() {
        Token::Lambda => {
            let next = it.next().ok_or(ParserBodyError::UnexpectedEof)?;
            let ident = if let Token::Ident = next.0.unwrap() {
                src[next.1].to_string()
            } else {
                return Err(ParserBodyError::IdentNotFound);
            };
            if it.next().ok_or(ParserBodyError::UnexpectedEof)?.0.unwrap() != Token::Dot {
                return Err(ParserBodyError::MissingDot);
            }
            let b = UBody::Abs(ident, try_from_iter(it, src)?);
            Ok(UTerm::from(b))
        }
        Token::OpenParen => {
            let val = try_from_iter(it, src)?;
            if it.next().ok_or(ParserBodyError::UnexpectedEof)?.0.unwrap() != Token::CloseParen {
                return Err(ParserBodyError::ParenUnclosed);
            }
            Ok(val)
        }
        Token::Ident => {
            let ident = src[span.clone()].to_string();
            Ok(UTerm::from(UBody::Var(ident)))
        }
        _ => Err(ParserBodyError::UnexpectedToken(tok.clone())),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser;
    use church::{assert_alpha_eq, assert_alpha_ne, Body, Term};

    fn assert_ast_eq(lhs: &str, rhs: &str) {
        assert_alpha_eq!(
            Term::try_from(parser::try_from_str(lhs).unwrap()).unwrap(),
            Term::try_from(parser::try_from_str(rhs).unwrap()).unwrap()
        )
    }

    fn assert_ast_ne(lhs: &str, rhs: &str) {
        assert_alpha_ne!(
            Term::try_from(parser::try_from_str(lhs).unwrap()).unwrap(),
            Term::try_from(parser::try_from_str(rhs).unwrap()).unwrap()
        )
    }

    #[test]
    fn id() {
        let parsed: Term = parser::try_from_str("λx.x").unwrap().try_into().unwrap();
        let built: Term = Body::Abs(0, Body::Var(0).into()).into();
        assert_alpha_eq!(built, parsed);
    }

    #[test]
    fn greedy_expr() {
        assert_ast_eq("λx.x λy.y x", "λx . (x λy.y x)");
    }

    #[test]
    fn right_assoc_app() {
        assert_ast_eq("a b (c d) e f", "(((a b) (c d)) e) f");
        assert_ast_ne("a b (c d) e f", "a (((b (c d)) e) f)");
    }
}
