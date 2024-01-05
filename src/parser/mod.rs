use crate::{Body, VarId};
use logos::Logos;
use lrp::{Clr, Dfa, Meta, Parser, Token};

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord)]
pub enum Ast {
    Expr(Body),
    Token(Sym),
    Var(VarId),
}

impl Ast {
    pub fn as_expr(&self) -> &Body {
        match self {
            Self::Expr(e) => e,
            Self::Token(_) => unreachable!(),
            Self::Var(_) => unreachable!(),
        }
    }

    pub fn as_var(&self) -> VarId {
        match self {
            Self::Expr(_) => unreachable!(),
            Self::Token(_) => unreachable!(),
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
    Expr,
    Token,
    EntryPoint,
    Lambda,
    App,
    Eof,
}

pub fn lexer(src: &str) -> impl Iterator<Item = Gramem> + '_ {
    Sym::lexer(src).spanned().map(|(t, s)| {
        let ast = match t.as_ref().expect("invalid symbol") {
            Sym::Var => {
                let s = &src[s.start..s.end];
                let post = (s.len() - 1) * crate::ALPHABET.len();
                let c = s.chars().next().unwrap() as usize - 'a' as usize;
                Ast::Var(post + c)
            }
            _ => Ast::Token(t.unwrap()),
        };
        Token::new(Meta::new(ast, s.into()), t.unwrap())
    })
}

pub mod out;

#[must_use]
pub fn build_parser<I: Iterator<Item = Gramem>>(buffer: I) -> Dfa<Meta<Ast>, Sym, I> {
    let parser = Clr::new(out::grammar());
    parser.dfa(buffer, out::reduct_map())
}

pub fn parse<I: Iterator<Item = Gramem>>(buffer: I) -> Result<Body, lrp::Error<Sym>> {
    let mut parser = build_parser(buffer);
    match parser.start() {
        Err(e) => Err(e),
        Ok(..) => Ok(parser.items[0].item.item.as_expr().clone()),
    }
}
