use std::str::FromStr;

use parser::ParserBodyError;

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

impl FromStr for UTerm {
    type Err = ParserBodyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::try_from_str(s)
    }
}
