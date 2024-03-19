pub mod ubody;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub parser);

use ubody::UnprocessedBody;

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord)]
pub enum Ast {
    Expr(UnprocessedBody),
}

impl Ast {
    pub fn into_ubody(self) -> Box<UnprocessedBody> {
        assert!(matches!(self, Self::Expr(..)));
        match self {
            Self::Expr(e) => Box::new(e),
            _ => unreachable!(),
        }
    }
}
