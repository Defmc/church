use core::fmt;
use std::{collections::HashMap, iter::Peekable};

pub type VarId = usize;
pub type FnId = usize;

pub const ALPHABET: &str = "abcdefghijklmnopqrtstuvwxyz";
pub fn id_to_str(i: usize) -> &'static str {
    &ALPHABET[i % ALPHABET.len()..i % ALPHABET.len() + 1]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn from_args<I: Iterator<Item = VarId>>(mut it: Peekable<I>, body: Body) -> Option<Self> {
        let next = it.next()?;
        let body = if it.peek().is_some() {
            let abs = Self::from_args(it, body).unwrap();
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

    fn redex_by_alpha(&mut self, map: &mut HashMap<VarId, VarId>) {
        if map.contains_key(&self.var) {
            let map_len = map.len();
            let mut map = map.clone();
            map.insert(self.var, map_len);
            self.var = map_len;
            map.insert(self.var, map_len);
            self.body.redex_by_alpha(&mut map);
        } else {
            map.insert(self.var, map.len());
            self.var = map.len() - 1;
            self.body.redex_by_alpha(map)
        }
    }

    pub fn alpha_eq(&self, rhs: &Self) -> bool {
        self.eq_by_alpha(rhs, &mut HashMap::new(), &mut HashMap::new())
    }

    pub fn eq_by_alpha(
        &self,
        rhs: &Self,
        self_map: &mut HashMap<VarId, VarId>,
        rhs_map: &mut HashMap<VarId, VarId>,
    ) -> bool {
        if self_map.contains_key(&self.var) {
            let map_len = self_map.len();
            let mut map = self_map.clone();
            map.insert(self.var, map_len);
            return self.eq_by_alpha(&rhs, &mut map, rhs_map);
        } else {
            self_map.insert(self.var, self_map.len());
        }
        if rhs_map.contains_key(&rhs.var) {
            let map_len = rhs_map.len();
            let mut map = rhs_map.clone();
            map.insert(rhs.var, map_len);
            return self.eq_by_alpha(&rhs, self_map, &mut map);
        } else {
            rhs_map.insert(rhs.var, rhs_map.len());
        }
        self.body.eq_by_alpha(&rhs.body, self_map, rhs_map)
    }

    pub fn apply(self, val: &Body) -> Body {
        let id = self.var;
        self.applied(id, val).body
    }

    pub fn applied(mut self, id: VarId, val: &Body) -> Self {
        self.body.apply(id, val);
        self
    }

    pub fn curry(&mut self, val: &Body) -> &mut Self {
        let id = self.var;
        self.body.apply(id, val);
        if let Body::Abs(l) = &self.body {
            *self = *l.clone();
        } else {
            unreachable!()
        }
        self
    }

    pub fn beta_redex(&mut self) {
        self.body.beta_redex()
    }
}

impl fmt::Display for Lambda {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("λ{}.{}", id_to_str(self.var), self.body))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Body {
    /* identity */ Id(VarId),
    /* application */ App(Box<Body>, /* ( */ Box<Body> /* ) */),
    /* abstraction */ Abs(Box<Lambda>),
}

impl Body {
    pub fn redex_by_alpha(&mut self, map: &mut HashMap<VarId, VarId>) {
        match self {
            Self::Id(id) => *id = map[id], // TODO: allow free variables
            Self::App(f, x) => {
                f.redex_by_alpha(map);
                x.redex_by_alpha(map);
            }
            Self::Abs(l) => l.redex_by_alpha(map),
        }
    }

    pub fn eq_by_alpha(
        &self,
        rhs: &Self,
        self_map: &mut HashMap<VarId, VarId>,
        rhs_map: &mut HashMap<VarId, VarId>,
    ) -> bool {
        match (self, rhs) {
            (Self::Id(s_id), Self::Id(r_id)) => self_map[s_id] == rhs_map[r_id],
            (Self::App(s_f, s_x), Self::App(r_f, r_x)) => {
                s_f.eq_by_alpha(r_f, self_map, rhs_map) && s_x.eq_by_alpha(r_x, self_map, rhs_map)
            }
            (Self::Abs(s_l), Self::Abs(r_l)) => s_l.eq_by_alpha(r_l, self_map, rhs_map),
            (_, _) => false,
        }
    }

    pub fn apply(&mut self, id: VarId, val: &Self) {
        match self {
            Self::Id(s_id) => {
                if *s_id == id {
                    *self = val.clone()
                }
            }
            Self::Abs(l) => l.body.apply(id, val),
            Self::App(f, x) => {
                f.apply(id, val);
                x.apply(id, val);
            }
        }
    }

    pub fn beta_redex(&mut self) {
        match self {
            Self::Id(..) => {}
            Self::App(f, x) => {
                f.beta_redex();
                x.beta_redex();
                if let Self::Abs(l) = f.as_mut() {
                    *self = l.clone().apply(x);
                }
            }
            Self::Abs(l) => {
                l.beta_redex();
            }
        }
    }
}

impl fmt::Display for Body {
    fn fmt(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(id) => w.write_fmt(format_args!("{}", id_to_str(*id))),
            Self::App(ref f, ref x) => w.write_fmt(format_args!("({f} {x})")),
            Self::Abs(l) => w.write_fmt(format_args!("{l}")),
        }
    }
}

impl From<Lambda> for Body {
    fn from(value: Lambda) -> Self {
        Self::Abs(value.into())
    }
}

#[macro_export]
macro_rules! as_var_id {
    ($var:ident) => {
        ('z' as usize - (stringify!($var).as_bytes()[0] as usize))
    };
}

#[macro_export]
macro_rules! lambda {
    (^$var:ident $($body:tt)+) => {
        Body::Abs(Lambda {
            var: as_var_id!($var),
            body: lambda!($($body)+),
        }.into())
    };
    ($first:tt $second:tt $($body:tt)+) => {
        Body::App(
            lambda!($first $second).into(),
            lambda!($($body)+).into()
        )
    };
    ($first:tt ($second:tt $($body:tt)+)) => {
        Body::App(
            lambda!($first).into(),
            lambda!($second $($body)+).into()
        )
    };
    (($first:tt $second:tt) $($body:tt)+) => {
        lambda!($first $second $($body)+)
    };
    ($first:tt $second:tt ($($body:tt)+)) => {
        lambda!($first $second $($body)+)
    };
    (($first:tt $second:tt) ($($body:tt)+)) => {
        lambda!($first $second $($body)+)
    };
    (($first:tt) $second:tt) => {
        lambda!($first $second)
    };
    ($first:tt ($second:tt)) => {
        lambda!($first $second)
    };
    (($first:tt) ($second:tt)) => {
        lambda!($first $second)
    };
    ($first:tt $second:tt) => {
        Body::App(
            lambda!($first).into(),
            lambda!($second).into()
        )
    };
    ($first:tt) => {
        Body::Id(as_var_id!($first))
    };
    (($first:tt)) => {
        lambda!($first)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{Body, Lambda, VarId};

    fn flip(y_id: VarId, x_id: VarId, f_id: VarId) -> Lambda {
        // flip f x y = f y x
        // flip = ^f^x^y . (f y x)
        let fy /* f -> y -> x -> (fy -> x) */ = Body::App(
            Body::Id(f_id).into(),
            Body::Id(y_id).into(),
            );
        let body = Body::App(fy.into(), Body::Id(x_id).into());
        Lambda::from_args([f_id, x_id, y_id].into_iter().peekable(), body).unwrap()
    }

    #[test]
    fn flip_format() {
        assert_eq!(flip(0, 1, 2).to_string(), "λc.λb.λa.((c a) b)");
    }

    #[test]
    fn id_format() {
        assert_eq!(Lambda::id().to_string(), "λa.a");
    }

    #[test]
    fn flip_alpha_redex() {
        let mut flip = flip(VarId::MAX, VarId::MAX / 2, 0);
        flip.alpha_redex();
        assert_eq!(flip.to_string(), "λa.λb.λc.((a c) b)");
    }

    #[test]
    fn flip_alpha_eq() {
        let flip = flip(VarId::MAX, VarId::MAX / 2, 0);
        let alpha_redexed = {
            let mut flip = flip.clone();
            flip.alpha_redex();
            flip
        };
        assert!(flip.alpha_eq(&alpha_redexed));
    }

    #[test]
    fn flip_application() {
        let mut flip = flip(1, 2, 3);
        flip.alpha_redex();

        assert_eq!(flip.to_string(), "λa.λb.λc.((a c) b)");
        flip.curry(&Body::Id(5));
        assert_eq!(flip.to_string(), "λb.λc.((f c) b)");
        flip.curry(&Body::Id(6));
        assert_eq!(flip.to_string(), "λc.((f c) g)");
        let body = flip.apply(&Body::Id(7));
        assert_eq!(body.to_string(), "((f h) g)");
    }

    #[test]
    fn id_of_id_reduction() {
        const F_ID: usize = 1;
        const X_ID: usize = F_ID + 1;
        // id_f = ^f^x . f x
        let mut id_f = Lambda::from_args(
            [F_ID, X_ID].into_iter().peekable(),
            Body::App(Body::Id(F_ID).into(), Body::Id(X_ID).into()),
        )
        .unwrap();
        // id = ^x . x
        let id = Lambda::id();
        id_f.curry(&Body::Abs(id.into()));
        id_f.beta_redex();
        assert!(id_f.alpha_eq(&Lambda::id()));
    }

    #[test]
    fn id_id_alpha_redex() {
        let mut abs = Lambda {
            var: 0,
            body: Body::App(Body::Abs(Lambda::id().into()).into(), Body::Id(0).into()),
        };
        abs.alpha_redex();
    }

    #[test]
    fn flip_flip_alpha_redex() {
        let mut fliper = flip(0, 1, 2);
        let fliper_f = flip(0, 1, 2);
        fliper.curry(&fliper_f.into());
        println!("{fliper}");
        fliper.alpha_redex();
        println!("{fliper}");
    }
}
