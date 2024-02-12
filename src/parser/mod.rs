use crate::{Term, VarId};
use logos::Logos;
use lrp::{Dfa, Meta, Parser, Slr, Token};

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq)]
pub enum Ast {
    Expr(Term),
    Token(Sym),
    Var(VarId),
}

impl Ast {
    #[must_use]
    pub fn as_expr(&self) -> &Term {
        match self {
            Self::Expr(e) => e,
            Self::Token(_) | Self::Var(_) => unreachable!(),
        }
    }

    #[must_use]
    pub fn as_var(&self) -> VarId {
        match self {
            Self::Expr(_) | Self::Token(_) => unreachable!(),
            Self::Var(id) => *id,
        }
    }
}

pub type Gramem = Token<Meta<Ast>, Sym>;

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord, Logos, Copy)]
pub enum Sym {
    #[token("λ")]
    #[token("^")]
    #[token("\\")]
    LambdaChar,
    #[regex(r#"[a-z]'*"#)]
    Var,
    #[token(".")]
    #[token("->")]
    Body,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[regex(r"[ \t\n\r]+", logos::skip)]
    Ws,
    Unknown((usize, usize)),
    Expr,
    Token,
    EntryPoint,
    Lambda,
    App,
    Eof,
}

/// # Panics
/// Always where is a invalid symbol
pub fn lexer(src: &str) -> impl Iterator<Item = Gramem> + '_ {
    Sym::lexer(src).spanned().map(|(t, s)| {
        let ast = match t.as_ref() {
            Ok(Sym::Var) => {
                let s = &src[s.start..s.end];
                let post = (s.len() - 1) * crate::ALPHABET.len();
                let c = s.chars().next().unwrap() as usize - 'a' as usize;
                Ast::Var(post + c)
            }
            Ok(_) => Ast::Token(t.unwrap()),
            Err(_) => panic!("invalid symbol {:?}", &src[s.start..s.end]),
        };
        Token::new(Meta::new(ast, s.into()), t.unwrap())
    })
}

pub fn try_lexer(src: &str) -> Result<impl Iterator<Item = Gramem> + '_, lrp::Error<Sym>> {
    let mut v = Vec::new();
    for (t, s) in Sym::lexer(src).spanned() {
        let ast = match t.as_ref() {
            Ok(Sym::Var) => {
                let s = &src[s.start..s.end];
                let post = (s.len() - 1) * crate::ALPHABET.len();
                let c = s.chars().next().unwrap() as usize - 'a' as usize;
                Ast::Var(post + c)
            }
            Ok(_) => Ast::Token(t.unwrap()),
            Err(_) => {
                return Err(lrp::Error::UnexpectedToken(
                    Sym::Unknown((s.start, s.end)),
                    vec![],
                ))
            }
        };
        v.push(Token::new(Meta::new(ast, s.into()), t.unwrap()))
    }
    Ok(v.into_iter())
}

pub mod out;

#[must_use]
pub fn build<I: Iterator<Item = Gramem>>(buffer: I) -> Dfa<Meta<Ast>, Sym, I> {
    let parser = Slr::new(out::grammar());
    parser.dfa(buffer, out::reduct_map())
}

/// # Errors
/// Same as `lrp::Dfa` and `lexer`.
pub fn parse<I: Iterator<Item = Gramem>>(buffer: I) -> Result<Term, lrp::Error<Sym>> {
    let mut parser = build(buffer);
    match parser.start() {
        Err(e) => Err(e),
        Ok(..) => Ok(parser.items[0].item.item.as_expr().clone().updated()),
    }
}
