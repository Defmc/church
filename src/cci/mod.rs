use lalrpop_util::lalrpop_mod;
use std::sync::OnceLock;

lalrpop_mod!(pub parser);
pub mod mode;
pub mod runner;
pub mod scope;
pub mod ubody;

use ubody::UnprocessedBody;

pub static GLOBAL_PROGRAM_PARSER: OnceLock<parser::ProgramParser> = OnceLock::new();

pub fn get_global_parser() -> &'static parser::ProgramParser {
    GLOBAL_PROGRAM_PARSER.get_or_init(|| parser::ProgramParser::new())
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord)]
pub enum Ast {
    Expr(Box<UnprocessedBody>),
    LetExpr(String, Box<UnprocessedBody>),
    Program(Vec<Box<Self>>),
}

impl Ast {
    pub fn into_ubody(self) -> Box<UnprocessedBody> {
        assert!(matches!(self, Self::Expr(..)));
        match self {
            Self::Expr(e) => e,
            _ => unreachable!(),
        }
    }

    pub fn into_program(self) -> Vec<Box<Self>> {
        assert!(matches!(self, Self::Program(..)));
        match self {
            Self::Program(v) => v,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::get_global_parser;

    #[test]
    fn lambda_expressions() {
        assert!(get_global_parser().parse("^a.(a)").is_ok());
        assert!(get_global_parser().parse("^Param.(Function Param)").is_ok());
        assert!(get_global_parser().parse("Just a TEST").is_ok());
        assert!(get_global_parser().parse("Const").is_ok());
    }

    #[test]
    fn let_expressions() {
        assert!(get_global_parser().parse("Const = Value").is_ok());
        assert!(get_global_parser().parse("Sum = FoldR Add 0").is_ok());
    }
}
