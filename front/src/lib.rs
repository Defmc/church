use std::path::PathBuf;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar);

#[derive(Debug)]
pub enum Ast {
    Program(Vec<Ast>),
    Assign(String, Box<Ast>),
    Let(Vec<Ast>, Box<Ast>),
    Use(String),

    // basic term
    App(Box<Ast>, Box<Ast>),
    Abs(String, Box<Ast>),
    Var(String),
}

use church::Term;
use thiserror::Error;

pub mod cu;
pub mod former;
pub mod parser;
pub mod scope;

#[derive(Error, Debug)]
pub enum Error {
    #[error("couldn't find module {0}")]
    ModuleNotFound(PathBuf),

    #[error("Definition for `{0}` wasn't found")]
    DefNotFound(String),

    #[error("{0:?}")]
    ParserError(parser::Error),

    #[error("{0:?}")]
    LexerError(()),

    #[error("Variable {0}'ve been already deifned as {1}")]
    AlreadyDefined(String, Term),
}

pub type Result<T> = std::result::Result<T, Error>;
