use std::str::FromStr;

use lalrpop_util::{lalrpop_mod, ParseError};
lalrpop_mod!(pub grammar);

pub enum Ast {
    Program(Vec<Ast>),
    Assign(String, Box<Ast>),
    Expr(UTerm),
}

use church::Term;
use scope::Scope;

pub mod former;
pub mod parser;
pub mod scope;

#[derive(Debug, Clone)]
pub enum UBody {
    Var(String),
    App(UTerm, UTerm),
    Abs(String, UTerm),
}

#[derive(Debug, Clone)]
pub struct UTerm {
    pub body: Box<UBody>,
}

impl From<UBody> for UTerm {
    fn from(value: UBody) -> Self {
        Self { body: value.into() }
    }
}

impl TryFrom<UBody> for Term {
    type Error = scope::Error;
    fn try_from(value: UBody) -> Result<Self, Self::Error> {
        UTerm::from(value).try_into()
    }
}

impl TryFrom<UTerm> for Term {
    type Error = scope::Error;
    fn try_from(value: UTerm) -> Result<Self, Self::Error> {
        Scope::default().dump(&value)
    }
}

impl FromStr for UTerm {
    type Err = parser::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::try_from_str(s)
    }
}
