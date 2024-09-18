use std::ops::Range;

use lalrpop_util::ParseError;
use logos::Logos;

#[derive(Logos, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
#[logos(skip r"#.*[\n#]")]
#[logos(skip r"[ \f]+")]
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

    #[regex("[a-zA-Z0-9α-κμ-ωΑ-ΚΜ-Ω_]+", |lex| lex.slice().to_string())]
    Ident(String),

    #[token("\n")]
    NewLine,

    #[token("\t")]
    Tab,

    #[token("=")]
    Assign,

    #[token("use")]
    UseKw,

    #[token("let")]
    LetKw,

    #[token("in")]
    InKw,

    #[token(",")]
    Comma,

    #[regex(r#"\"(?:[^\\"]|\\\\|\\")*\""#, |lex| lex.slice()[1..lex.slice().len() - 1].to_string())]
    Path(String),
}

impl Token {
    pub fn rebuild_code(tokens: &[ParserToken]) -> String {
        let mut buf = String::new();
        for (_, tk, _) in tokens {
            match tk {
                Self::Dot => buf.push('.'),
                Self::Lambda => buf.push('λ'),
                Self::Tab => buf.push('\t'),
                Self::NewLine => buf.push('\n'),
                Self::LetKw => buf.push_str("let"),
                Self::InKw => buf.push_str("in"),
                Self::UseKw => buf.push_str("use"),
                Self::Comma => buf.push(','),
                Self::Assign => buf.push('='),
                Self::Ident(id) => buf.push_str(id),
                Self::Path(p) => buf.push_str(p),
                Self::OpenParen => buf.push('('),
                Self::CloseParen => buf.push(')'),
            }
            buf.push(' ');
        }
        buf
    }
}

pub type ParserToken = (usize, Token, usize);
pub type LexerTy = (std::result::Result<Token, ()>, Range<usize>);
pub type Result<T> = std::result::Result<T, ParseError<usize, Token, ()>>;
pub type Error = lalrpop_util::ParseError<usize, Token, ()>;

// #[cfg(test)]
// mod tests {
//     use crate::parser;
//     use church::{assert_alpha_eq, assert_alpha_ne, Body, Term};
//
//     fn assert_ast_eq(lhs: &str, rhs: &str) {
//         assert_alpha_eq!(
//             Term::try_from(parser::try_from_str(lhs).unwrap()).unwrap(),
//             Term::try_from(parser::try_from_str(rhs).unwrap()).unwrap()
//         )
//     }
//
//     fn assert_ast_ne(lhs: &str, rhs: &str) {
//         assert_alpha_ne!(
//             Term::try_from(parser::try_from_str(lhs).unwrap()).unwrap(),
//             Term::try_from(parser::try_from_str(rhs).unwrap()).unwrap()
//         )
//     }
//
//     #[test]
//     fn id() {
//         let parsed: Term = parser::try_from_str("λx.x").unwrap().try_into().unwrap();
//         let built: Term = Body::Abs(0, Body::Var(0).into()).into();
//         assert_alpha_eq!(built, parsed);
//     }
//
//     #[test]
//     fn greedy_expr() {
//         assert_ast_eq("λx.x λy.y x", "λx . (x λy.y x)");
//     }
//
//     #[test]
//     fn right_assoc_app() {
//         assert_ast_eq("a b (c d) e f", "(((a b) (c d)) e) f");
//         assert_ast_ne("a b (c d) e f", "a (((b (c d)) e) f)");
//     }
// }
