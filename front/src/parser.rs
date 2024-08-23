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
    let (tok, span) = it.next().ok_or(ParserBodyError::UnexpectedEof)?;
    let expr = match tok.as_ref().unwrap() {
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
            return Ok(UTerm::from(b));
        }
        Token::OpenParen => {
            let val = try_from_iter(it, src)?;
            if it.next().ok_or(ParserBodyError::UnexpectedEof)?.0.unwrap() != Token::CloseParen {
                return Err(ParserBodyError::ParenUnclosed);
            }
            val
        }
        Token::Ident => {
            let ident = src[span].to_string();
            UTerm::from(UBody::Var(ident))
        }
        _ => {
            return Err(ParserBodyError::UnexpectedToken(tok));
        }
    };
    match it.peek().map(|(t, _)| t) {
        Some(Ok(Token::Lambda)) | None => Ok(expr),
        _ => {
            let rhs = try_from_iter(it, src)?;
            let b = UBody::App(expr, rhs);
            Ok(UTerm::from(b))
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{parser, UBody, UTerm};
//
//     use church::{assert_alpha_eq, assert_alpha_ne};
//
//     #[test]
//     fn id() {
//         let parsed = parser::try_from_str("λx.x").unwrap();
//         let built: UTerm = UBody::Abs(0, UBody::Var(0).into()).into();
//         assert_alpha_eq!(built, parsed);
//     }
//
//     #[test]
//     fn ambiguous_expr() {
//         const EQUIVALENTS: &[&str] = &["λx.x λy.y x", "λx . (x λy.y x)"];
//         const DIFFERENTS: &[&str] = &["(λx.x) (λy.y) x"];
//         for e in EQUIVALENTS {
//             assert_alpha_eq!(
//                 parser::try_from_str(EQUIVALENTS[0]).unwrap(),
//                 parser::try_from_str(e).unwrap()
//             );
//         }
//         for d in DIFFERENTS {
//             assert_alpha_ne!(
//                 parser::try_from_str(EQUIVALENTS[0]).unwrap(),
//                 parser::try_from_str(d).unwrap()
//             );
//         }
//     }
// }
