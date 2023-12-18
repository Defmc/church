use core::fmt;
use std::collections::HashMap;

pub type VarId = usize;
pub type FnId = usize;

pub const ALPHABET: &str = "abcdefghijklmnopqrtstuvwxyz";
pub fn alpha_alias(i: usize) -> &'static str {
    &ALPHABET[i % ALPHABET.len()..i % ALPHABET.len() + 1]
}

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
        // TODO: avoid clone by using a `Peekable` iterator
        let next = it.next()?;
        let body = if let Some(abs) = Self::from_args(it, body.clone()) {
            Body::Abs(abs.into())
        } else {
            body
        };
        let l = Lambda::new(next, body);
        Some(l)
    }

    pub fn alpha_redex(&mut self) {
        self.redex_by_alpha(&mut HashMap::new())
    }

    fn redex_by_alpha(&mut self, map: &mut HashMap<usize, usize>) {
        assert!(
            !map.contains_key(&self.var),
            "shadowing {}",
            alpha_alias(self.var)
        );
        map.insert(self.var, map.len());
        self.var = map.len() - 1;
        self.body.redex_by_alpha(map)
    }

    pub fn alpha_eq(&self, rhs: &Self) -> bool {
        self.eq_by_alpha(rhs, &mut HashMap::new(), &mut HashMap::new())
    }

    pub fn eq_by_alpha(
        &self,
        rhs: &Self,
        self_map: &mut HashMap<usize, usize>,
        rhs_map: &mut HashMap<usize, usize>,
    ) -> bool {
        assert!(
            !self_map.contains_key(&self.var),
            "shadowing {} in self",
            alpha_alias(self.var)
        );
        assert!(
            !rhs_map.contains_key(&rhs.var),
            "shadowing {} in rhs",
            alpha_alias(rhs.var)
        );
        self_map.insert(self.var, self_map.len());
        rhs_map.insert(rhs.var, rhs_map.len());
        self.body.eq_by_alpha(&rhs.body, self_map, rhs_map)
    }
}

impl fmt::Display for Lambda {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("λ{}.{}", alpha_alias(self.var), self.body))
    }
}

#[derive(Debug, Clone)]
pub enum Body {
    /* identity */ Id(VarId),
    /* application */ App(Box<Body>, /* ( */ Box<Body> /* ) */),
    /* abstraction */ Abs(Box<Lambda>),
}

impl Body {
    pub fn redex_by_alpha(&mut self, map: &mut HashMap<VarId, VarId>) {
        match self {
            Body::Id(id) => *id = map[id],
            Body::App(f, x) => {
                f.redex_by_alpha(map);
                x.redex_by_alpha(map);
            }
            Body::Abs(l) => l.redex_by_alpha(map),
        }
    }

    pub fn eq_by_alpha(
        &self,
        rhs: &Self,
        self_map: &mut HashMap<VarId, VarId>,
        rhs_map: &mut HashMap<VarId, VarId>,
    ) -> bool {
        println!("self_map: {self_map:?}\nrhs_map: {rhs_map:?}");
        match (self, rhs) {
            (Body::Id(s_id), Body::Id(r_id)) => self_map[s_id] == rhs_map[r_id],
            (Body::App(s_f, s_x), Body::App(r_f, r_x)) => {
                s_f.eq_by_alpha(r_f, self_map, rhs_map) && s_x.eq_by_alpha(r_x, self_map, rhs_map)
            }
            (Body::Abs(s_l), Body::Abs(r_l)) => s_l.eq_by_alpha(r_l, self_map, rhs_map),
            (_, _) => false,
        }
    }
}

impl fmt::Display for Body {
    fn fmt(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Body::Id(id) => w.write_fmt(format_args!("{}", alpha_alias(*id))),
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

    #[test]
    fn flip_alpha_redex() {
        // flip f x y = f y x
        // flip = ^f^x^y . (f y x)
        const Y_ID: VarId = usize::MAX;
        const X_ID: VarId = usize::MAX / 2;
        const F_ID: VarId = 0;
        let fy /* f -> y -> x -> (fy -> x) */ = Body::App(
            Body::Id(F_ID).into(),
            Body::Id(Y_ID).into(),
            );
        let body = Body::App(fy.into(), Body::Id(X_ID).into());
        let mut flip = Lambda::from_args([F_ID, X_ID, Y_ID].into_iter().peekable(), body).unwrap();
        flip.alpha_redex();
        assert_eq!(flip.to_string(), "λa.λb.λc.((a c) b)");
    }

    #[test]
    fn flip_alpha_eq() {
        // flip f x y = f y x
        // flip = ^f^x^y . (f y x)
        const Y_ID: VarId = 3;
        const X_ID: VarId = 4;
        const F_ID: VarId = 5;
        let fy /* f -> y -> x -> (fy -> x) */ = Body::App(
            Body::Id(F_ID).into(),
            Body::Id(Y_ID).into(),
            );
        let body = Body::App(fy.into(), Body::Id(X_ID).into());
        let flip = Lambda::from_args([F_ID, X_ID, Y_ID].into_iter().peekable(), body).unwrap();
        let alpha_redexed = {
            let mut flip = flip.clone();
            flip.alpha_redex();
            flip
        };
        assert!(flip.alpha_eq(&alpha_redexed));
    }
}
