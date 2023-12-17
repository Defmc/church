use core::fmt;
use std::{collections::HashMap, iter::Peekable, num::NonZeroUsize};

pub type VarId = usize;

pub const ALPHABET: &str = "abcdefghijklmnopqrtstuvwxyzabcdefghijklmnopqrtstuvwxyz";

/// Church encoding
pub mod enc;

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

    /// Returns the order (how many abstractions) this term have
    #[must_use]
    pub fn order(&self) -> usize {
        match self {
            Self::Abs(_, ref a) => 1 + a.order(),
            _ => 0,
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

    #[must_use]
    pub fn alpha_reduced(mut self) -> Self {
        self.alpha_redex();
        self
    }

    #[must_use]
    pub fn beta_reduced(mut self) -> Self {
        self.beta_redex();
        self
    }

    /// # Panics
    /// Never.
    pub fn redex_by_alpha(&mut self, map: &mut HashMap<VarId, VarId>) {
        match self {
            Self::Id(id) => {
                if let Some(mid) = map.get(id) {
                    *id = *mid;
                }
            }
            Self::App(f, x) => {
                f.redex_by_alpha(map);
                x.redex_by_alpha(map);
            }
            Self::Abs(i, l) => {
                if map.contains_key(i) {
                    let mut map = map.clone();
                    *map.get_mut(i).unwrap() = map.len();
                    *i = map.len();
                    map.insert(map.len(), *i);
                    l.redex_by_alpha(&mut map);
                } else {
                    map.insert(*i, map.len());
                    *i = map.len() - 1;
                    l.redex_by_alpha(map);
                }
            }
        }
    }

    /// α-equivalency refers to expressions with same implementation, disconsidering the variable
    /// names. `PartialEq` compares the variables, so we can say that `PartialEq` ⊂ `alpha_eq`.
    /// ^f^x . f x != ^g^y . g y, but
    /// ^f^x . f x α== ^g^y . g y, where
    /// ^f^x . f (f x) α!= ^f^x . f x and ^f^x . f (f x) != ^f^x . f x
    #[must_use]
    pub fn alpha_eq(&self, rhs: &Self) -> bool {
        self.eq_by_alpha(rhs, &mut HashMap::new(), &mut HashMap::new())
    }

    /// # Panics
    /// Never.
    pub fn eq_by_alpha(
        &self,
        rhs: &Self,
        self_map: &mut HashMap<VarId, VarId>,
        rhs_map: &mut HashMap<VarId, VarId>,
    ) -> bool {
        match (self, rhs) {
            (Self::Id(s_id), Self::Id(r_id)) => self_map.get(s_id) == rhs_map.get(r_id),
            (Self::App(s_f, s_x), Self::App(r_f, r_x)) => {
                s_f.eq_by_alpha(r_f, self_map, rhs_map) && s_x.eq_by_alpha(r_x, self_map, rhs_map)
            }
            (Self::Abs(s_v, s_l), Self::Abs(r_v, r_l)) => {
                let mut edits = (None, None);
                if self_map.contains_key(s_v) {
                    let mut map = self_map.clone();
                    *map.get_mut(s_v).unwrap() = map.len();
                    map.insert(map.len(), *s_v);
                    edits.0 = Some(map);
                } else {
                    self_map.insert(*s_v, self_map.len());
                }
                if rhs_map.contains_key(r_v) {
                    let mut map = rhs_map.clone();
                    *map.get_mut(r_v).unwrap() = map.len();
                    map.insert(map.len(), *r_v);
                    edits.1 = Some(map);
                } else {
                    rhs_map.insert(*r_v, rhs_map.len());
                }
                s_l.eq_by_alpha(
                    r_l,
                    edits.0.as_mut().map_or_else(|| self_map, |m| m),
                    edits.1.as_mut().map_or_else(|| rhs_map, |m| m),
                )
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
            Self::Abs(v, l) => {
                if *v != id {
                    l.apply_by(id, val);
                }
            }
            Self::App(f, x) => {
                f.apply_by(id, val);
                x.apply_by(id, val);
            }
        }
    }

    /// "Curries" the function, turning it into the next term
    /// `curry` (^x.^f . f x) c == ^f . f c
    /// # Panics
    /// If it isn't a lambda abstraction.
    pub fn curry(&mut self, val: &Self) -> &mut Self {
        let (v, l) = self.as_mut_abs().expect("currying a non-abstraction");
        l.apply_by(*v, val);
        *self = *l.clone();
        self
    }

    #[must_use]
    pub fn applied<'a>(mut self, vals: impl IntoIterator<Item = &'a Self>) -> Self {
        for v in vals {
            self.curry(v);
        }
        self
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

    /// Returns the length of expressions as the amount of variables related.
    /// len(a) == 1
    /// len(^a.a) == 2
    /// len(^f^x^y . f x y) == 6
    /// # Panics
    /// Never.
    #[must_use]
    pub fn len(&self) -> NonZeroUsize {
        match self {
            Self::Id(..) => 1.try_into().unwrap(),
            Self::App(ref f, ref x) => f.len().saturating_add(x.len().into()),
            Self::Abs(_, ref b) => b.len().saturating_add(1),
        }
    }

    /// Declarative alternative for `Self::from_args`
    /// # Panics
    /// Never.
    #[must_use]
    pub fn with(self, it: impl IntoIterator<Item = VarId>) -> Self {
        Self::from_args(it.into_iter().peekable(), self).unwrap()
    }
}

impl fmt::Display for Body {
    fn fmt(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(id) => w.write_fmt(format_args!("{}", id_to_str(*id))),
            Self::App(ref f, ref x) => w.write_fmt(format_args!(
                "{f} {}",
                if usize::from(x.len()) > 1 {
                    format!("({x})")
                } else {
                    format!("{x}")
                }
            )),
            Self::Abs(v, l) => w.write_fmt(format_args!("λ{}.{l}", id_to_str(*v))),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{enc::natural, Body, VarId};

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
        assert_eq!(flip(0, 1, 2).to_string(), "λc.λb.λa.c a b");
    }

    #[test]
    fn id_format() {
        assert_eq!(Body::id().to_string(), "λa.a");
    }

    #[test]
    fn flip_alpha_redex() {
        let mut flip = flip(VarId::MAX, VarId::MAX / 2, 0);
        flip.alpha_redex();
        assert_eq!(flip.to_string(), "λa.λb.λc.a c b");
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

        assert_eq!(flip.to_string(), "λa.λb.λc.a c b");
        flip.curry(&Body::Id(5));
        assert_eq!(flip.to_string(), "λb.λc.f c b");
        flip.curry(&Body::Id(6));
        assert_eq!(flip.to_string(), "λc.f c g");
        let body = flip.curry(&Body::Id(7));
        assert_eq!(body.to_string(), "f h g");
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

    #[test]
    fn right_associative_format() {
        const F_ID: VarId = 0;
        const X_ID: VarId = 1;
        assert_eq!(natural(F_ID, X_ID, 0).to_string(), "λa.λb.b");
        assert_eq!(natural(F_ID, X_ID, 1).to_string(), "λa.λb.a b");
        assert_eq!(
            natural(F_ID, X_ID, 5).to_string(),
            "λa.λb.a (a (a (a (a b))))"
        );
        assert_eq!(
            natural(F_ID, X_ID, 10).to_string(),
            "λa.λb.a (a (a (a (a (a (a (a (a (a b)))))))))"
        );
    }
}
