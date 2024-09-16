use std::{path::PathBuf, str::FromStr};

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar);

#[derive(Debug)]
pub enum Ast {
    Program(Vec<Ast>),
    Assign(String, UTerm),
    Use(String),
}

use church::Term;
use scope::Scope;
use thiserror::Error;

pub mod cu;
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
    type Error = Error;
    fn try_from(value: UBody) -> std::result::Result<Self, Self::Error> {
        UTerm::from(value).try_into()
    }
}

impl TryFrom<UTerm> for Term {
    type Error = Error;
    fn try_from(value: UTerm) -> std::result::Result<Self, Self::Error> {
        Scope::default().dump(&value)
    }
}

impl FromStr for UTerm {
    type Err = parser::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        parser::try_from_str(s)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("couldn't find module {0}")]
    ModuleNotFound(PathBuf),

    #[error("Definition for `{0}` wasn't found")]
    DefNotFound(String),

    #[error("{0:?}")]
    ParserError(parser::Error),

    #[error("Variable {0}'ve been already deifned as {1}")]
    AlreadyDefined(String, Term),
}

pub type Result<T> = std::result::Result<T, Error>;
