use std::iter::Peekable;

use crate::{Body, Term};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParserBodyError {
    MissingDot,
    UnexpectedChar(char),
    UnexpectedEof,
    ParenUnclosed,
}

pub type Result<T> = std::result::Result<T, ParserBodyError>;

pub fn try_from_str(s: &str) -> Result<Term> {
    try_from_iter(&mut s.chars().filter(|c| c != &' ').peekable())
}

pub fn try_from_iter<I>(it: &mut Peekable<I>) -> Result<Term>
where
    I: Iterator<Item = char>,
{
    match it.next().ok_or(ParserBodyError::UnexpectedEof)? {
        'λ' => {
            let ident = it.next().ok_or(ParserBodyError::UnexpectedEof)?;
            if it.next().ok_or(ParserBodyError::UnexpectedEof)? != '.' {
                return Err(ParserBodyError::MissingDot);
            }
            let b = Body::Abs(ident as usize, try_from_iter(it)?);
            Ok(Term::from(b))
        }
        '(' => {
            let val = try_from_iter(it)?;
            if it.next().ok_or(ParserBodyError::UnexpectedEof)? != ')' {
                return Err(ParserBodyError::ParenUnclosed);
            }
            Ok(val)
        }
        c => {
            let ident = c;
            let b = Body::Var(ident as usize);
            let var = Term::from(b);
            match it.peek() {
                Some(')') | None => Ok(var),
                _ => {
                    let b = Body::App(var, try_from_iter(it)?);
                    Ok(Term::from(b))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{assert_alpha_eq, assert_alpha_ne, Body, Term};

    #[test]
    fn id() {
        let parsed = Term::from_str("λx.x").unwrap();
        let built: Term = Body::Abs(0, Body::Var(0).into()).into();
        assert_alpha_eq!(built, parsed);
    }

    #[test]
    fn ambiguous_expr() {
        const EQUIVALENTS: &[&str] = &["λx.x λy.y x", "λx . (x λy.y x)"];
        const DIFFERENTS: &[&str] = &["(λx.x) (λy.y) x"];
        for e in EQUIVALENTS {
            assert_alpha_eq!(
                Term::from_str(EQUIVALENTS[0]).unwrap(),
                Term::from_str(e).unwrap()
            );
        }
        for d in DIFFERENTS {
            assert_alpha_ne!(
                Term::from_str(EQUIVALENTS[0]).unwrap(),
                Term::from_str(d).unwrap()
            );
        }
    }
}
