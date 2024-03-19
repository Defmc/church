pub mod ubody;
use lalrpop_util::lalrpop_mod;
use std::sync::OnceLock;

lalrpop_mod!(pub parser);

use ubody::UnprocessedBody;

pub static GLOBAL_PROGRAM_PARSER: OnceLock<parser::ProgramParser> = OnceLock::new();

pub fn get_global_parser() -> &'static parser::ProgramParser {
    GLOBAL_PROGRAM_PARSER.get_or_init(|| parser::ProgramParser::new())
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord)]
pub enum Ast {
    Expr(Box<UnprocessedBody>),
    LetExpr(String, Box<UnprocessedBody>),
}

impl Ast {
    pub fn into_ubody(self) -> Box<UnprocessedBody> {
        assert!(matches!(self, Self::Expr(..)));
        match self {
            Self::Expr(e) => e,
            _ => unreachable!(),
        }
    }
}
