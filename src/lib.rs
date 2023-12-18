use core::fmt;
use std::iter::Peekable;

pub type VarId = usize;
pub type FnId = usize;

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


#[derive(Debug, Clone)]
pub enum Body {
    /* identity */ Id(VarId),
    /* application */ App(Box<Body>, /* ( */ Box<Body> /* ) */),
    /* abstraction */ Abs(Box<Lambda>),
}


