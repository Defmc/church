use core::fmt;
use std::num::NonZeroUsize;

use logos::Logos;
use lrp::{Dfa, Meta, Parser, Slr, Token};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{id_to_str, Body, Term, VarId};

pub mod out;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum UnprocessedBody {
    Var(String),
    Abs(String, Box<Self>),
    App(Box<Self>, Box<Self>),
}

impl UnprocessedBody {
    pub fn dump(&self) -> Term {
        let mut set = HashSet::default();
        self.get_used_vars(&mut set);

        let renames: HashMap<String, VarId> = set
            .iter()
            .cloned()
            .zip(Self::get_free_names(&set))
            .collect();
        self.dump_with(&renames)
    }

    pub fn dump_with(&self, map: &HashMap<String, VarId>) -> Term {
        match self {
            Self::Var(s) => Term::new(Body::Id(map[s])),
            Self::App(lhs, rhs) => Term::new(Body::App(lhs.dump_with(map), rhs.dump_with(map))),
            Self::Abs(arg, fun) => Term::new(Body::Abs(map[arg], fun.dump_with(map))),
        }
    }

    pub fn get_used_vars(&self, set: &mut HashSet<String>) {
        match self {
            Self::Var(s) => {
                if !set.contains(s) {
                    set.insert(s.clone());
                }
            }
            Self::Abs(arg, fun) => {
                if !set.contains(arg) {
                    set.insert(arg.clone());
                }
                fun.get_used_vars(set);
            }
            Self::App(lhs, rhs) => {
                lhs.get_used_vars(set);
                rhs.get_used_vars(set);
            }
        }
    }

    pub fn get_free_names(used_names: &HashSet<String>) -> impl Iterator<Item = VarId> + '_ {
        (0..).filter(|&i| !used_names.contains(&id_to_str(i)))
    }

    #[must_use]
    pub fn len(&self) -> NonZeroUsize {
        match *self {
            Self::Var(..) => 1.try_into().unwrap(),
            Self::App(ref f, ref x) => f.len().saturating_add(x.len().into()),
            Self::Abs(_, ref b) => b.len().saturating_add(1),
        }
    }

    pub fn try_from_str<T: AsRef<str>>(s: T) -> Result<Self, lrp::Error<Sym>> {
        let lex = try_lexer(s.as_ref())?;
        parse(lex)
    }
}

impl fmt::Display for UnprocessedBody {
    fn fmt(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(id) => w.write_fmt(format_args!("{id}")),
            Self::App(ref f, ref x) => w.write_fmt(format_args!(
                "{f} {}",
                if usize::from(x.len()) > 1 {
                    format!("({x})")
                } else {
                    format!("{x}")
                }
            )),
            Self::Abs(v, l) => w.write_fmt(format_args!("λ{v}.({l})")),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord)]
pub enum Ast {
    Expr(UnprocessedBody),
    Token(Sym),
    Var(String),
}

impl Ast {
    #[must_use]
    pub fn as_expr(&self) -> &UnprocessedBody {
        match self {
            Self::Expr(e) => e,
            Self::Token(_) | Self::Var(_) => unreachable!(),
        }
    }

    #[must_use]
    pub fn into_boxed_expr(&self) -> Box<UnprocessedBody> {
        Box::new(self.as_expr().clone())
    }

    #[must_use]
    pub fn as_var(&self) -> &str {
        match self {
            Self::Expr(_) | Self::Token(_) => unreachable!(),
            Self::Var(id) => id,
        }
    }

    #[must_use]
    pub fn into_var(&self) -> String {
        self.as_var().to_owned()
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
        Ok(..) => Ok(parser.items[0].item.item.as_expr().clone()),
    }
}
