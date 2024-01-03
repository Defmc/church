use crate::Body;
use logos::Logos;

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord, Logos)]
pub enum Sym {
    #[token("λ")]
    #[token("^")]
    #[token("\\")]
    Lambda,
    #[regex(r#"[a-zA-Z_]\w*"#)]
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
}
