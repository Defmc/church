use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    iter::Peekable,
    num::NonZeroUsize,
    str::FromStr,
};

pub type VarId = usize;

pub const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";

/// Church encoding
pub mod enc;

/// Parsing lib
pub mod parser;

/// # Panics
/// Never.
#[must_use]
pub fn id_to_str(i: usize) -> String {
    let rotations = i / ALPHABET.len();
    let i = i % ALPHABET.len();
    format!(
        "{}{}",
        ALPHABET[i..=i].chars().next().unwrap(),
        "'".repeat(rotations)
    )
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

    pub fn as_mut_abs(&mut self) -> Option<(&mut VarId, &mut Self)> {
        if let Self::Abs(v, b) = self {
            Some((v, b))
        } else {
            None
        }
    }

    #[must_use]
    pub fn in_app(self, s: Self) -> Body {
        Body::App(self.into(), s.into())
    }

    pub fn alpha_redex(&mut self) {
        let frees = self.free_variables().into_iter();
        self.redex_by_alpha(&mut frees.map(|i| (i, i)).collect())
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
                f.redex_by_alpha(&mut map.clone());
                x.redex_by_alpha(&mut map.clone());
            }
            Self::Abs(i, l) => {
                let (mut maybe_map, bind) = Self::try_alpha_redex(*i, map);
                *i = bind;
                if let Some(ref mut new_map) = maybe_map {
                    l.redex_by_alpha(new_map)
                } else {
                    l.redex_by_alpha(map)
                }
            }
        }
    }

    /// returns all free variables, including the ones binded on one application
    /// FV(^x.(x a x) b) = { a, b }
    /// FV(^x.(a) ^a.(a)) = { a }
    /// FV(a) = { a }
    #[must_use]
    pub fn free_variables(&self) -> HashSet<VarId> {
        let (mut binds, mut frees) = (HashSet::new(), HashSet::new());
        self.get_free_variables(&mut binds, &mut frees);
        frees
    }

    /// return all variables used
    /// FV(^x.(x a x) b) = { x, a, b }
    /// FV(^x.(a) ^a.(a)) = { x, a }
    /// FV(a) = { a }
    #[must_use]
    pub fn variables(&self) -> HashSet<VarId> {
        let mut binds = HashSet::new();
        self.get_variables(&mut binds);
        binds
    }

    fn get_variables(&self, binds: &mut HashSet<VarId>) {
        match self {
            Self::Id(id) => {
                binds.insert(*id);
            }
            Self::App(lhs, rhs) => {
                lhs.get_variables(binds);
                rhs.get_variables(binds);
            }
            Self::Abs(v, l) => {
                binds.insert(*v);
                l.get_variables(binds);
            }
        }
    }

    #[must_use]
    pub fn bounded_variables(&self) -> HashSet<VarId> {
        let mut binds = HashSet::new();
        self.get_bounded_variables(&mut binds);
        binds
    }

    fn get_bounded_variables(&self, binds: &mut HashSet<VarId>) {
        match self {
            Self::Id(..) => (),
            Self::App(lhs, rhs) => {
                lhs.get_bounded_variables(binds);
                rhs.get_bounded_variables(binds);
            }
            Self::Abs(v, l) => {
                binds.insert(*v);
                l.get_bounded_variables(binds);
            }
        }
    }

    pub fn get_free_variables(&self, binds: &mut HashSet<VarId>, frees: &mut HashSet<VarId>) {
        match self {
            Self::Id(id) => {
                if !binds.contains(id) {
                    frees.insert(*id);
                }
            }
            Self::App(lhs, rhs) => {
                lhs.get_free_variables(&mut binds.clone(), frees);
                rhs.get_free_variables(&mut binds.clone(), frees);
            }
            Self::Abs(v, l) => {
                binds.insert(*v);
                l.get_free_variables(binds, frees)
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
        let mut self_frees: HashMap<_, _> =
            self.free_variables().into_iter().map(|i| (i, i)).collect();
        let self_binds = self_frees.len();
        let mut rhs_frees: HashMap<_, _> =
            rhs.free_variables().into_iter().map(|i| (i, i)).collect();
        let rhs_binds = rhs_frees.len();
        self.eq_by_alpha(rhs, &mut self_frees, self_binds, &mut rhs_frees, rhs_binds)
    }

    // TODO: Restore state instead of cloning the entire map
    #[must_use]
    pub fn try_alpha_redex(
        id: VarId,
        map: &mut HashMap<VarId, VarId>,
    ) -> (Option<HashMap<VarId, VarId>>, VarId) {
        let mut new_id = id;
        // TODO: Use a better data structure
        let keys: HashSet<VarId> = map.keys().copied().collect();
        while keys.contains(&new_id) || map.contains_key(&new_id) {
            new_id += 1;
        }
        if map.contains_key(&id) {
            let mut map = map.clone();
            *map.get_mut(&id).unwrap() = new_id;
            map.insert(new_id, new_id);
            (Some(map), new_id)
        } else {
            map.insert(id, new_id);
            map.insert(new_id, new_id);
            (None, new_id)
        }
    }

    /// # Panics
    /// Never.
    pub fn eq_by_alpha(
        &self,
        rhs: &Self,
        self_map: &mut HashMap<VarId, VarId>,
        self_binds: usize,
        rhs_map: &mut HashMap<VarId, VarId>,
        rhs_binds: usize,
    ) -> bool {
        match (self, rhs) {
            (Self::Id(s_id), Self::Id(r_id)) => self_map.get(s_id) == rhs_map.get(r_id),
            (Self::App(s_f, s_x), Self::App(r_f, r_x)) => {
                s_f.eq_by_alpha(
                    r_f,
                    &mut self_map.clone(),
                    self_binds,
                    &mut rhs_map.clone(),
                    rhs_binds,
                ) && s_x.eq_by_alpha(
                    r_x,
                    &mut self_map.clone(),
                    self_binds,
                    &mut rhs_map.clone(),
                    rhs_binds,
                )
            }
            (Self::Abs(s_v, s_l), Self::Abs(r_v, r_l)) => {
                let mut edits = (None, None);
                if self_map.contains_key(s_v) {
                    let mut map = self_map.clone();
                    *map.get_mut(s_v).unwrap() = self_binds;
                    edits.0 = Some(map);
                } else {
                    self_map.insert(*s_v, self_binds);
                }
                if rhs_map.contains_key(r_v) {
                    let mut map = rhs_map.clone();
                    *map.get_mut(r_v).unwrap() = rhs_binds;
                    edits.1 = Some(map);
                } else {
                    rhs_map.insert(*r_v, rhs_binds);
                }
                s_l.eq_by_alpha(
                    r_l,
                    edits.0.as_mut().map_or_else(|| self_map, |m| m),
                    self_binds + 1,
                    edits.1.as_mut().map_or_else(|| rhs_map, |m| m),
                    rhs_binds + 1,
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
            Self::Abs(..) => {
                let v = *self.as_mut_abs().unwrap().0;
                if v != id {
                    self.fix_captures(val);
                    self.as_mut_abs().unwrap().1.apply_by(id, val);
                }
            }
            Self::App(f, x) => {
                f.apply_by(id, val);
                x.apply_by(id, val);
            }
        }
    }

    pub fn fix_captures(&mut self, rhs: &Self) {
        let vars = self.bounded_variables();
        let frees_val = rhs.free_variables();
        // if there's no free variable capturing (used vars on lhs /\ free vars on rhs), we just apply on the abstraction body
        let mut captures: Vec<_> = frees_val.intersection(&vars).collect(); // TODO: Use vec
        captures.sort();
        if !captures.is_empty() {
            // println!("debugging {self} and {rhs}");
            // println!(
            //     "frees_val: {:?}",
            //     frees_val
            //         .iter()
            //         .map(|s| id_to_str(*s))
            //         .collect::<HashSet<_>>()
            // );
            // println!(
            //     "vars: {:?}",
            //     vars.iter().map(|s| id_to_str(*s)).collect::<HashSet<_>>()
            // );
            // println!(
            //     "captures founded: {:?}",
            //     captures
            //         .iter()
            //         .map(|s| id_to_str(**s))
            //         .collect::<HashSet<_>>()
            // );
            let reserveds: HashSet<_> = rhs.variables().union(&self.variables()).copied().collect();
            let safes = (0..).filter(|n| !reserveds.contains(n)).take(vars.len());
            let news = safes.zip(&captures);
            for (to, from) in news {
                // println!("origin: {self}");
                self.rename_vars(**from, to, false);
            }
            // println!("final: {self} | {rhs}");
        }
    }

    /// renames a bounded variable over an expression, fixing the bind abstraction.
    /// `force` makes all ocurrences of to be renamed, desconsidering the context.
    /// rename_vars ^x.(x y) 'x' 'a' false = ^a.(a y)
    pub fn rename_vars(&mut self, from: VarId, to: VarId, force: bool) {
        match self {
            Self::Id(ref mut i) => {
                assert_ne!(*i, to);
                if *i == from && force {
                    *i = to;
                }
            }
            Self::Abs(ref mut v, ref mut l) => {
                assert_ne!(*v, to);
                let force = *v == from || force;
                if *v == from && force {
                    *v = to;
                }
                l.rename_vars(from, to, force);
            }
            Self::App(ref mut lhs, ref mut rhs) => {
                lhs.rename_vars(from, to, force);
                rhs.rename_vars(from, to, force);
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
        *self = l.clone();
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
                if matches!(f.as_ref(), Self::Abs(..)) {
                    let mut f = f.clone();
                    f.fix_captures(x);
                    let (id, l) = f.as_mut_abs().unwrap();
                    l.apply_by(*id, x);
                    *self = l.clone();
                    self.beta_redex();
                }
            }
            Self::Abs(..) => {
                if self.eta_redex_step() {
                    self.beta_redex();
                } else if let Self::Abs(_, l) = self {
                    l.beta_redex();
                }
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

    pub fn eta_redex_step(&mut self) -> bool {
        if let Self::Abs(v, app) = self {
            if let Self::App(lhs, rhs) = app.as_ref() {
                if rhs.as_ref() == &Self::Id(*v) && !lhs.contains(&Body::Id(*v)) {
                    *self = *lhs.clone();
                    return true;
                }
            }
        }
        false
    }

    /// returns if, at any depth, starting from the outmost expression, there's the passed
    /// expression.
    pub fn contains(&self, what: &Self) -> bool {
        match self {
            Self::Id(..) => self == what,
            Self::App(lhs, rhs) => {
                self == lhs.as_ref()
                    || self == rhs.as_ref()
                    || lhs.contains(what)
                    || rhs.contains(what)
            }
            Self::Abs(_, l) => self == what || l.contains(what),
        }
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
            Self::Abs(v, l) => w.write_fmt(format_args!("λ{}.({l})", id_to_str(*v))),
        }
    }
}

impl FromStr for Body {
    type Err = lrp::Error<parser::Sym>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lex = parser::lexer(s);
        parser::parse(lex)
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use crate::{enc::naturals::natural, Body, VarId};

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
        assert_eq!(flip(0, 1, 2).to_string(), "λc.(λb.(λa.(c a b)))");
    }

    #[test]
    fn id_format() {
        assert_eq!(Body::id().to_string(), "λa.(a)");
    }

    #[test]
    fn flip_alpha_redex() {
        let mut flip = flip(10, 5, 0);
        flip.alpha_redex();
        assert!(
            flip.alpha_eq(&Body::from_str("λa.(λb.(λc.(a c b)))").unwrap()),
            "{flip}"
        );
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

        assert!(flip.alpha_eq(&Body::from_str("λa.(λb.(λc.(a c b)))").unwrap()));
        flip.curry(&Body::Id(5));
        assert!(flip.alpha_eq(&Body::from_str("λb.(λc.(f c b))").unwrap()));
        flip.curry(&Body::Id(6));
        assert!(flip.alpha_eq(&Body::from_str("λc.(f c g)").unwrap()));
        let body = flip.curry(&Body::Id(7));
        assert!(body.alpha_eq(&Body::from_str("f h g").unwrap()));
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
        assert!(id_f.alpha_eq(&Body::id()), "{id_f}");
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
        fliper.curry(&fliper_f);
        fliper.alpha_redex();
        fliper.beta_redex();
    }

    #[test]
    fn right_associative_format() {
        assert_eq!(natural(0).to_string(), "λa.(λb.(b))");
        assert_eq!(natural(1).to_string(), "λa.(λb.(a b))");
        assert_eq!(natural(5).to_string(), "λa.(λb.(a (a (a (a (a b))))))");
        assert_eq!(
            natural(10).to_string(),
            "λa.(λb.(a (a (a (a (a (a (a (a (a (a b)))))))))))"
        );
    }

    #[test]
    pub fn free_capture_avoiding_subsitution() {
        let expr = Body::from_str("λb.(λa.(b a a)) λx.(a)").unwrap();
        assert!(expr
            .beta_reduced()
            .alpha_eq(&Body::from_str("λc.(a c)").unwrap()));
    }
    #[test]
    pub fn bound_capture_avoiding_subsitution() {
        let expr = Body::from_str("λa.(λb.(λa.(b a a)) λx.(a))").unwrap();
        assert!(expr
            .beta_reduced()
            .alpha_eq(&Body::from_str("λa.(λc.(a c))").unwrap()));
    }
}
