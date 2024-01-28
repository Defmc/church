use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    iter::Peekable,
    num::NonZeroUsize,
    str::FromStr,
};

pub type VarId = usize;

pub const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";

/// Parsing lib
pub mod parser;

/// Delta Reductions
pub mod scope;

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
pub struct Term {
    pub body: Body,
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.body.fmt(f)
    }
}

impl Term {
    pub fn new(body: Body) -> Self {
        Self { body }
    }

    #[must_use]
    pub fn id() -> Self {
        let id = Self::new(Body::Id(0));
        Self::new(Body::Abs(0, id.into()))
    }

    /// Create a lambda abstraction from left-to-right arguments
    /// `from_args` f x y (f x y) = ^f^x^y . f x y
    /// # Panics
    /// Never.
    pub fn from_args<I: Iterator<Item = VarId>>(mut it: Peekable<I>, term: Self) -> Option<Self> {
        let next = it.next()?;
        if it.peek().is_some() {
            let abs = Self::from_args(it, term).unwrap();
            let body = Self::new(abs.body);
            let abs = Body::Abs(next, body.into());
            Some(Self::new(abs))
        } else {
            let abs = Body::Abs(next, term.into());
            Some(Self::new(abs))
        }
    }

    /// Returns the order (how many abstractions) this body have
    #[must_use]
    pub fn order(&self) -> usize {
        match self.body {
            Body::Abs(_, ref a) => 1 + a.order(),
            _ => 0,
        }
    }

    pub fn as_mut_abs(&mut self) -> Option<(&mut VarId, &mut Self)> {
        if let Body::Abs(ref mut v, ref mut b) = self.body {
            Some((v, b.as_mut()))
        } else {
            None
        }
    }

    pub fn as_mut_app(&mut self) -> Option<(&mut Self, &mut Self)> {
        if let Body::App(ref mut lhs, ref mut rhs) = self.body {
            Some((lhs.as_mut(), rhs))
        } else {
            None
        }
    }

    pub fn as_mut_id(&mut self) -> Option<&mut VarId> {
        if let Body::Id(ref mut id) = self.body {
            Some(id)
        } else {
            None
        }
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
        match self.body {
            Body::Id(ref mut id) => {
                if let Some(mid) = map.get(id) {
                    *id = *mid;
                }
            }
            Body::App(ref mut f, ref mut x) => {
                f.redex_by_alpha(&mut map.clone());
                x.redex_by_alpha(&mut map.clone());
            }
            Body::Abs(ref mut i, ref mut l) => {
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
        match &self.body {
            Body::Id(id) => {
                binds.insert(*id);
            }
            Body::App(lhs, rhs) => {
                lhs.get_variables(binds);
                rhs.get_variables(binds);
            }
            Body::Abs(v, l) => {
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
        match &self.body {
            Body::Id(..) => (),
            Body::App(lhs, rhs) => {
                lhs.get_bounded_variables(binds);
                rhs.get_bounded_variables(binds);
            }
            Body::Abs(v, l) => {
                binds.insert(*v);
                l.get_bounded_variables(binds);
            }
        }
    }

    pub fn get_free_variables(&self, binds: &mut HashSet<VarId>, frees: &mut HashSet<VarId>) {
        match &self.body {
            Body::Id(id) => {
                if !binds.contains(&id) {
                    frees.insert(*id);
                }
            }
            Body::App(lhs, rhs) => {
                lhs.get_free_variables(&mut binds.clone(), frees);
                rhs.get_free_variables(&mut binds.clone(), frees);
            }
            Body::Abs(v, l) => {
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
        match (&self.body, &rhs.body) {
            (Body::Id(s_id), Body::Id(r_id)) => self_map.get(&s_id) == rhs_map.get(&r_id),
            (Body::App(s_f, s_x), Body::App(r_f, r_x)) => {
                s_f.eq_by_alpha(
                    &r_f,
                    &mut self_map.clone(),
                    self_binds,
                    &mut rhs_map.clone(),
                    rhs_binds,
                ) && s_x.eq_by_alpha(
                    &r_x,
                    &mut self_map.clone(),
                    self_binds,
                    &mut rhs_map.clone(),
                    rhs_binds,
                )
            }
            (Body::Abs(s_v, s_l), Body::Abs(r_v, r_l)) => {
                let mut edits = (None, None);
                if self_map.contains_key(&s_v) {
                    let mut map = self_map.clone();
                    *map.get_mut(&s_v).unwrap() = self_binds;
                    edits.0 = Some(map);
                } else {
                    self_map.insert(*s_v, self_binds);
                }
                if rhs_map.contains_key(&r_v) {
                    let mut map = rhs_map.clone();
                    *map.get_mut(&r_v).unwrap() = rhs_binds;
                    edits.1 = Some(map);
                } else {
                    rhs_map.insert(*r_v, rhs_binds);
                }
                s_l.eq_by_alpha(
                    &r_l,
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
        match self.body {
            Body::Id(s_id) => {
                if s_id == id {
                    *self = val.clone();
                }
            }
            Body::Abs(..) => {
                let v = *self.as_mut_abs().unwrap().0;
                if v != id {
                    self.fix_captures(val);
                    self.as_mut_abs().unwrap().1.apply_by(id, val);
                }
            }
            Body::App(ref mut f, ref mut x) => {
                f.apply_by(id, val);
                x.apply_by(id, val);
            }
        }
    }

    pub fn fix_captures(&mut self, rhs: &Self) {
        let vars = self.bounded_variables();
        let frees_val = rhs.free_variables();
        // if there's no free variable capturing (used vars on lhs /\ free vars on rhs), we just apply on the abstraction body
        let captures: Vec<_> = frees_val.intersection(&vars).collect(); // TODO: Use vec
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
            self.redex_by_alpha(&mut captures.into_iter().map(|&i| (i, i)).collect());
            // println!("final: {self} | {rhs}");
        }
    }

    /// renames a bounded variable over an expression, fixing the bind abstraction.
    /// `force` makes all ocurrences of to be renamed, desconsidering the context.
    /// rename_vars ^x.(x y) 'x' 'a' false = ^a.(a y)
    pub fn rename_vars(&mut self, from: VarId, to: VarId, force: bool) {
        match self.body {
            Body::Id(ref mut i) => {
                assert_ne!(*i, to);
                if *i == from && force {
                    *i = to;
                }
            }
            Body::Abs(ref mut v, ref mut l) => {
                assert_ne!(*v, to);
                let force = *v == from || force;
                if *v == from && force {
                    *v = to;
                }
                l.rename_vars(from, to, force);
            }
            Body::App(ref mut lhs, ref mut rhs) => {
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
        while self.beta_redex_step() {}
    }

    pub fn beta_redex_step(&mut self) -> bool {
        match self.body {
            Body::Id(..) => false,
            Body::App(ref mut f, ref mut x) => {
                return if matches!(&f.body, Body::Abs(..)) {
                    let mut f = f.clone();
                    f.fix_captures(&x);
                    let (id, l) = f.as_mut_abs().unwrap();
                    l.apply_by(*id, &x);
                    *self = l.clone();
                    true
                } else {
                    f.beta_redex_step() || x.beta_redex_step()
                };
            }
            Body::Abs(..) => {
                if self.eta_redex_step() {
                    true
                } else if let Body::Abs(_, ref mut l) = self.body {
                    l.beta_redex_step()
                } else {
                    unreachable!()
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
        match self.body {
            Body::Id(..) => 1.try_into().unwrap(),
            Body::App(ref f, ref x) => f.len().saturating_add(x.len().into()),
            Body::Abs(_, ref b) => b.len().saturating_add(1),
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
        if let Body::Abs(v, ref mut app) = self.body {
            if let Body::App(ref mut lhs, ref mut rhs) = app.body {
                if &rhs.body == &Body::Id(v) && !lhs.contains(&Body::Id(v)) {
                    *self = *lhs.clone();
                    return true;
                }
            }
        }
        false
    }

    /// returns if, at any depth, starting from the outmost expression, there's the passed
    /// expression.
    pub fn contains(&self, what: &Body) -> bool {
        match &self.body {
            Body::Id(..) => self.body == *what,
            Body::App(lhs, rhs) => {
                self.body == lhs.body
                    || self.body == rhs.body
                    || lhs.contains(what)
                    || rhs.contains(what)
            }
            Body::Abs(_, l) => self.body == *what || l.contains(what),
        }
    }
}

impl FromStr for Term {
    type Err = lrp::Error<parser::Sym>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lex = parser::lexer(s);
        parser::parse(lex)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Body {
    /* identity */ Id(VarId),
    /* application */ App(Box<Term>, /* ( */ Box<Term> /* ) */),
    /* abstraction */ Abs(VarId, Box<Term>),
}

impl Body {}

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

#[cfg(test)]
pub mod tests {}
