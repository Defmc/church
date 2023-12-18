use core::fmt;
use std::iter::Peekable;

pub type VarId = usize;
pub type FnId = usize;

pub const ALPHABET: &str = "abcdefghijklmnopqrtstuvwxyz";

#[derive(Debug, Clone)]
pub struct Lambda {
    pub var: VarId,
    pub body: Body,
}

impl Lambda {
    pub fn new(var: VarId, body: Body) -> Self {
        Self { var, body }
    }

    pub fn id() -> Self {
        Self {
            var: 0,
            body: Body::Id(0),
        }
    }

    pub fn from_args(mut it: impl Iterator<Item = VarId>, body: Body) -> Option<Self> {
        // TODO: alpha reduction
        let next = it.next()?;
        let body = if let Some(abs) = Self::from_args(it, body.clone()) {
            Body::Abs(abs.into())
        } else {
            body
        };
        let l = Lambda::new(next, body);
        Some(l)
    }
}

impl fmt::Display for Lambda {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "λ{}.{}",
            &ALPHABET[self.var % ALPHABET.len()..self.var % ALPHABET.len() + 1],
            self.body
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Body {
    /* identity */ Id(VarId),
    /* application */ App(Box<Body>, /* ( */ Box<Body> /* ) */),
    /* abstraction */ Abs(Box<Lambda>),
}

impl fmt::Display for Body {
    fn fmt(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Body::Id(id) => w.write_fmt(format_args!(
                "{}",
                &ALPHABET[id % ALPHABET.len()..id % ALPHABET.len() + 1]
            )),
            Body::App(ref f, ref x) => w.write_fmt(format_args!("({f} {x})")),
            Body::Abs(l) => w.write_fmt(format_args!("{l}")),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{Body, Lambda, VarId};

    #[test]
    fn flip() {
        // flip f x y = f y x
        // flip = ^f^x^y . (f y x)
        const Y_ID: VarId = 0;
        const X_ID: VarId = 1;
        const F_ID: VarId = 2;
        let fy /* f -> y -> x -> (fy -> x) */ = Body::App(
            Body::Id(F_ID).into(),
            Body::Id(Y_ID).into(),
            );
        let body = Body::App(fy.into(), Body::Id(X_ID).into());
        let flip = Lambda::from_args([F_ID, X_ID, Y_ID].into_iter().peekable(), body).unwrap();
        assert_eq!(flip.to_string(), "λc.λb.λa.((c a) b)");
    }

    #[test]
    fn id() {
        assert_eq!(Lambda::id().to_string(), "λa.a");
    }
}
