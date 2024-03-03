use logos::Logos;
use lrp::{Dfa, Meta, Parser, Slr, Token};
pub mod out;
pub mod ubody;

use ubody::UnprocessedBody;

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord)]
pub enum Ast {
    Expr(UnprocessedBody),
    Let(String, UnprocessedBody),
    Program(Vec<Self>),
    Token(Sym),
    Var(String),
}

impl Ast {
    #[must_use]
    pub fn as_expr(&self) -> &UnprocessedBody {
        match self {
            Self::Expr(e) => e,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn into_expr(self) -> UnprocessedBody {
        match self {
            Self::Expr(e) => e,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn into_boxed_expr(&self) -> Box<UnprocessedBody> {
        Box::new(self.as_expr().clone())
    }

    #[must_use]
    pub fn as_var(&self) -> &str {
        match self {
            Self::Var(id) => id,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn into_var(self) -> String {
        match self {
            Self::Var(id) => id,
            _ => unreachable!(),
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
    #[regex(r#"[a-zA-Z_]\w*"#)]
    Var,
    #[token(".")]
    #[token("->")]
    Body,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[regex(r"[ ]+", logos::skip)]
    Ws,
    #[token("=")]
    Let,
    Unknown((usize, usize)),
    Expr,
    Token,
    ProgramAtom,
    Program,
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
            Ok(Sym::Var) => Ast::Var(src[s.start..s.end].to_string()),
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
            Ok(Sym::Var) => Ast::Var(src[s.start..s.end].to_string()),
            Ok(_) => Ast::Token(t.unwrap()),
            Err(_) => {
                println!(
                    "[try_lexer] error: unexpected token {:?} ({}..{})\n[try_lexer] source: {src:?}",
                    &src[s.start..s.end],
                    s.start,
                    s.end
                );
                return Err(lrp::Error::UnexpectedToken(
                    Sym::Unknown((s.start, s.end)),
                    vec![],
                ));
            }
        };
        v.push(Token::new(Meta::new(ast, s.into()), t.unwrap()))
    }
    Ok(v.into_iter())
}

#[must_use]
pub fn build<I: Iterator<Item = Gramem>>(buffer: I) -> Dfa<Meta<Ast>, Sym, I> {
    let parser = Slr::new(out::grammar());
    parser.dfa(buffer, out::reduct_map())
}

/// # Errors
/// Same as `lrp::Dfa` and `lexer`.
pub fn parse<I: Iterator<Item = Gramem>>(buffer: I) -> Result<UnprocessedBody, lrp::Error<Sym>> {
    let mut parser = build(buffer);
    match parser.start() {
        Err(e) => Err(e),
        Ok(..) => Ok(parser.items.drain(..).next().unwrap().item.item.into_expr()),
    }
}
