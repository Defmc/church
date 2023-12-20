use core::fmt;
use std::{collections::HashMap, iter::Peekable};

pub type VarId = usize;
pub type FnId = usize;

pub const ALPHABET: &str = "abcdefghijklmnopqrtstuvwxyzabcdefghijklmnopqrtstuvwxyz";

#[must_use]
pub fn id_to_str(i: usize) -> &'static str {
    let rotations = i / ALPHABET.len();
    let i = i % ALPHABET.len();
    &ALPHABET[i..=i + rotations]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Body {
    /* identity */ Id(VarId),
    /* application */ App(Box<Body>, /* ( */ Box<Body> /* ) */),
    /* abstraction */ Abs(VarId, Box<Self>),
}

impl Body {
    #[must_use]
    pub fn id() -> Self {
        Self::Abs(0, Body::Id(0).into())
    }

    /// Create a lambda abstraction from left-to-right arguments
    /// `from_args` f x y (f x y) = ^f^x^y . f x y
    /// # Panics
    /// Never.
    pub fn from_args<I: Iterator<Item = VarId>>(mut it: Peekable<I>, body: Body) -> Option<Self> {
        let next = it.next()?;
        if it.peek().is_some() {
            let abs = Self::from_args(it, body).unwrap();
            Some(Body::Abs(next, abs.into()))
        } else {
            Some(Body::Abs(next, body.into()))
        }
    }

    pub fn as_mut_abs(&mut self) -> Option<(&mut VarId, &mut Box<Self>)> {
        if let Self::Abs(v, b) = self {
            Some((v, b))
        } else {
            None
        }
    }

    pub fn alpha_redex(&mut self) {
        self.redex_by_alpha(&mut HashMap::new());
    }

    pub fn redex_by_alpha(&mut self, map: &mut HashMap<VarId, VarId>) {
        match self {
            Self::Id(id) => *id = map[id], // TODO: allow free variables
            Self::App(f, x) => {
                f.redex_by_alpha(map);
                x.redex_by_alpha(map);
            }
            Self::Abs(i, l) => {
                if map.contains_key(i) {
                    let map_len = map.len();
                    let mut map = map.clone();
                    map.insert(*i, map_len);
                    *i = map_len;
                    map.insert(*i, map_len);
                    l.redex_by_alpha(&mut map);
                } else {
                    map.insert(*i, map.len());
                    *i = map.len() - 1;
                    l.redex_by_alpha(map);
                }
            }
        }
    }

    #[must_use]
    pub fn alpha_eq(&self, rhs: &Self) -> bool {
        self.eq_by_alpha(rhs, &mut HashMap::new(), &mut HashMap::new())
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
            (Self::Abs(s_v, s_l), Self::Abs(r_v, r_l)) => {
                if self_map.contains_key(s_v) {
                    let map_len = self_map.len();
                    let mut map = self_map.clone();
                    map.insert(*s_v, map_len);
                    return self.eq_by_alpha(rhs, &mut map, rhs_map);
                }
                self_map.insert(*s_v, self_map.len());
                if rhs_map.contains_key(r_v) {
                    let map_len = rhs_map.len();
                    let mut map = rhs_map.clone();
                    map.insert(*r_v, map_len);
                    return self.eq_by_alpha(rhs, self_map, &mut map);
                }
                rhs_map.insert(*r_v, rhs_map.len());
                s_l.eq_by_alpha(r_l, self_map, rhs_map)
            }
            (_, _) => false,
        }
    }

    pub fn apply_by(&mut self, id: VarId, val: &Self) {
        match self {
            Self::Id(s_id) => {
                if *s_id == id {
                    *self = val.clone();
                }
            }
            Self::Abs(_, l) => l.apply_by(id, val),
            Self::App(f, x) => {
                f.apply_by(id, val);
                x.apply_by(id, val);
            }
        }
    }

    /// Curry the function, mut don't need to have a next lambda abstraction
    /// `apply` (^f . f c) g == g c
    /// # Panics
    /// If it isn't a lambda abstraction
    #[must_use]
    pub fn apply(mut self, val: &Self) -> Self {
        if let Self::Abs(v, l) = &mut self {
            l.apply_by(*v, val);
            *l.clone()
        } else {
            assert!(matches!(self, Self::Abs(..)));
            unreachable!()
        }
    }

    /// "Curries" the function, turning it into the next lambda term
    /// `curry` (^x.^f . f x) c == ^f . f c
    /// # Panics
    /// If it isn't a lambda abstraction or if the next stage isn't a lambda abstraciton. In these cases, `Self::apply` should be used
    pub fn curry(&mut self, val: &Self) {
        if let Self::Abs(v, l) = self {
            l.apply_by(*v, val);
            *self = *l.clone();
        } else {
            assert!(matches!(self, Self::Abs(..)));
            unreachable!()
        };
        assert!(matches!(self, Self::Abs(..)));
    }

    pub fn beta_redex(&mut self) {
        match self {
            Self::Id(..) => {}
            Self::App(f, x) => {
                f.beta_redex();
                x.beta_redex();
                if let Self::Abs(v, l) = f.as_mut() {
                    let mut l = l.clone();
                    l.apply_by(*v, x);
                    *self = *l;
                }
            }
            Self::Abs(_, l) => {
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
            Self::Abs(v, l) => w.write_fmt(format_args!("λ{}.{l}", id_to_str(*v))),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{Body, VarId};

    fn flip(y_id: VarId, x_id: VarId, f_id: VarId) -> Body {
        // flip f x y = f y x
        // flip = ^f^x^y . (f y x)
        let fy /* f -> y -> x -> (fy -> x) */ = Body::App(
            Body::Id(f_id).into(),
            Body::Id(y_id).into(),
            );
        let body = Body::App(fy.into(), Body::Id(x_id).into());
        Body::from_args([f_id, x_id, y_id].into_iter().peekable(), body).unwrap()
    }

    #[test]
    fn flip_format() {
        assert_eq!(flip(0, 1, 2).to_string(), "λc.λb.λa.((c a) b)");
    }

    #[test]
    fn id_format() {
        assert_eq!(Body::id().to_string(), "λa.a");
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
        let mut id_f = Body::from_args(
            [F_ID, X_ID].into_iter().peekable(),
            Body::App(Body::Id(F_ID).into(), Body::Id(X_ID).into()),
        )
        .unwrap();
        // id = ^x . x
        let id = Body::id();
        id_f.curry(&id);
        id_f.beta_redex();
        assert!(id_f.alpha_eq(&Body::id()));
    }

    #[test]
    fn id_id_alpha_redex() {
        let mut abs = Body::Abs(0, Body::App(Body::id().into(), Body::Id(0).into()).into());
        abs.alpha_redex();
        abs.beta_redex();
        assert!(abs.alpha_eq(&Body::id()));
    }

    #[test]
    fn flip_flip_alpha_redex() {
        let mut fliper = flip(0, 1, 2);
        let fliper_f = {
            // flip f x y = f y x
            // flip = ^f^x^y . (f y x)
            const F_ID: usize = 2;
            const X_ID: usize = 1;
            const Y_ID: usize = 0;
            let fy /* f -> y -> x -> (fy -> x) */ = Body::App(
            Body::Id(F_ID).into(),
            Body::Id(Y_ID).into(),
            );
            let body = Body::App(fy.into(), Body::Id(X_ID).into());
            Body::from_args([X_ID, Y_ID, F_ID].into_iter().peekable(), body).unwrap()
        };
        fliper.curry(&fliper_f.into());
        println!("{fliper}");
        fliper.alpha_redex();
        println!("{fliper}");
        fliper.beta_redex();
        println!("{fliper}");
    }
}
