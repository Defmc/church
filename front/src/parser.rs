use std::ops::Range;

use lalrpop_util::ParseError;
use logos::Logos;

#[derive(Logos, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
#[logos(skip r"#.*[\n#]")]
#[logos(skip r"[ \f]+")]
pub enum Token {
    #[token("λ")]
    #[token("fn")]
    Lambda,

    #[token("=>")]
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
                Self::Dot => buf.push_str("=>"),
                Self::Lambda => buf.push_str("fn"),
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

#[cfg(test)]
mod tests {
    use crate::{cu::CodeUnit, grammar::ExprParser};
    use church::{assert_alpha_eq, assert_alpha_ne, Term};

    fn dump_expr(src: &str) -> Term {
        let tks = CodeUnit::into_tokens(src).unwrap();
        let expr = ExprParser::new().parse(tks).unwrap();
        let mut cu = CodeUnit::default();
        cu.scope.dump(&expr).unwrap()
    }

    fn assert_alpha_eq(lhs: &str, rhs: &str) {
        assert_alpha_eq!(dump_expr(lhs), dump_expr(rhs))
    }

    fn assert_alpha_ne(lhs: &str, rhs: &str) {
        assert_alpha_ne!(dump_expr(lhs), dump_expr(rhs))
    }

    #[test]
    fn greedy_expr() {
        assert_alpha_eq("λx => x λy => y x", "λx => (x λy => (y x))");
        assert_alpha_eq("λx => x λy => y x", "fn x => x fn y => y x");
    }

    #[test]
    fn right_assoc_app() {
        assert_alpha_eq("a b (c d) e f", "(((a b) (c d)) e) f");
        assert_alpha_ne("a b (c d) e f", "a (((b (c d)) e) f)");
    }
}
