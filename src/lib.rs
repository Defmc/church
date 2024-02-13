use core::fmt;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::rc::Rc;
use std::{iter::Peekable, num::NonZeroUsize, str::FromStr};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub body: Rc<Body>,
    /// closedness. A optimization tech.
    /// A term is called as `closed` if all variables used are declared inside the function. I.e:
    /// FV(x) == {} -> x is closed
    /// If x is closed and x = a b, then a and b are also closed,
    /// Any term can be not closed, but just closed terms CAN be closed
    pub closed: bool,
}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.body.partial_cmp(&other.body)
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.body.fmt(f)
    }
}

impl Term {
    pub fn new(body: Body) -> Self {
        Self::lazy_new(body).updated()
    }

    pub fn lazy_new(body: Body) -> Self {
        Self {
            body: Rc::new(body),
            closed: false, // there's no problem to use it as always `false`, but it will always
                           // check for free variables and handle it as it have. Performance will
                           // decrease for big expressions.
        }
    }

    pub fn updated(mut self) -> Self {
        self.update_closed();
        self
    }

    /// An optional function to optimize beta reductions. It's not needed, but guarantees to never
    /// check for free_variables in combinators. For example, in `Fact 6`, without `update_closed`,
    /// the computation takes 11s, while when enabled, it's 3.04s.
    pub fn update_closed(&mut self) {
        let mut frees = self.body.free_variables();
        let mut len = frees.len();
        self.set_closeds(&mut frees, &mut len);
    }

    pub fn set_closeds(&mut self, frees: &mut HashSet<VarId>, len: &mut usize) {
        self.closed = *len == 0;
        match Rc::make_mut(&mut self.body) {
            Body::Id(..) => (),
            Body::App(ref mut lhs, ref mut rhs) => {
                if !lhs.closed {
                    lhs.set_closeds(frees, len);
                }
                if !rhs.closed {
                    rhs.set_closeds(frees, len);
                }
            }
            Body::Abs(v, ref mut l) => {
                if !l.closed {
                    let old = frees.remove(v);
                    if old {
                        *len -= 1;
                    }
                    l.set_closeds(frees, len);
                    if old {
                        frees.insert(*v);
                        *len += 1;
                    }
                }
            }
        }
    }

    pub fn fast_inner_closed_check(&self) -> bool {
        match self.body.as_ref() {
            Body::Id(..) => false,
            Body::App(lhs, rhs) => lhs.closed && rhs.closed,
            Body::Abs(_, l) => l.closed,
        }
    }

    #[must_use]
    pub fn id() -> Self {
        let id = Self::new(Body::Id(0));
        Self::new(Body::Abs(0, id))
    }

    /// Create a lambda abstraction from left-to-right arguments
    /// `from_args` f x y (f x y) = ^f^x^y . f x y
    /// # Panics
    /// Never.
    pub fn from_args<I: Iterator<Item = VarId>>(mut it: Peekable<I>, term: Self) -> Option<Self> {
        let next = it.next()?;
        if it.peek().is_some() {
            let abs = Self::from_args(it, term).unwrap();
            let body = abs.clone();
            let abs = Body::Abs(next, body);
            Some(Self::new(abs))
        } else {
            let abs = Body::Abs(next, term);
            Some(Self::new(abs))
        }
    }

    /// Returns the order (how many abstractions) this body have
    #[must_use]
    pub fn order(&self) -> usize {
        match *self.body {
            Body::Abs(_, ref a) => 1 + a.order(),
            _ => 0,
        }
    }

    pub fn as_mut_abs(&mut self) -> Option<(&mut VarId, &mut Self)> {
        if let Body::Abs(ref mut v, ref mut b) = Rc::make_mut(&mut self.body) {
            Some((v, b))
        } else {
            None
        }
    }

    pub fn as_mut_app(&mut self) -> Option<(&mut Self, &mut Self)> {
        if let Body::App(ref mut lhs, ref mut rhs) = Rc::make_mut(&mut self.body) {
            Some((lhs, rhs))
        } else {
            None
        }
    }

    pub fn as_mut_id(&mut self) -> Option<&mut VarId> {
        if let Body::Id(ref mut id) = Rc::make_mut(&mut self.body) {
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
        match Rc::make_mut(&mut self.body) {
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

    #[must_use]
    pub fn bounded_variables(&self) -> HashSet<VarId> {
        self.body.bounded_variables()
    }

    /// returns all free variables, including the ones binded on one application
    /// FV(^x.(x a x) b) = { a, b }
    /// FV(^x.(a) ^a.(a)) = { a }
    /// FV(a) = { a }
    #[must_use]
    pub fn free_variables(&self) -> HashSet<VarId> {
        if self.closed {
            // debug_assert!(
            //     self.body.free_variables().is_empty(),
            //     "{self} is marked as closed, without being: {:?} are free",
            //     self.body.free_variables()
            // );
            // self.body.free_variables()
            HashSet::default()
        } else {
            self.body.free_variables()
        }
    }

    /// α-equivalency refers to expressions with same implementation, disconsidering the variable
    /// names. `PartialEq` compares the variables, so we can say that `PartialEq` ⊂ `alpha_eq`.
    /// ^f^x . f x != ^g^y . g y, but
    /// ^f^x . f x α== ^g^y . g y, where
    /// ^f^x . f (f x) α!= ^f^x . f x and ^f^x . f (f x) != ^f^x . f x
    #[must_use]
    pub fn alpha_eq(&self, rhs: &Self) -> bool {
        let mut self_frees: HashMap<_, _> = self.free_variables().iter().map(|&i| (i, i)).collect();
        let self_binds = self_frees.len();
        let mut rhs_frees: HashMap<_, _> = rhs.free_variables().iter().map(|&i| (i, i)).collect();
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
        match (self.body.as_ref(), rhs.body.as_ref()) {
            (Body::Id(s_id), Body::Id(r_id)) => self_map.get(s_id) == rhs_map.get(r_id),
            (Body::App(s_f, s_x), Body::App(r_f, r_x)) => {
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
            (Body::Abs(s_v, s_l), Body::Abs(r_v, r_l)) => {
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

    /// Replaces the terms with same `id` with `val`, returning if there was any application
    pub fn apply_by(&mut self, id: VarId, val: &Self) -> bool {
        let changed = match Rc::make_mut(&mut self.body) {
            Body::Id(s_id) => {
                if *s_id == id {
                    *self = val.clone();
                    true
                } else {
                    false
                }
            }
            Body::Abs(..) => {
                let v = *self.as_mut_abs().unwrap().0;
                if v != id {
                    self.fix_captures(val);
                    self.as_mut_abs().unwrap().1.apply_by(id, val)
                } else {
                    false
                }
            }
            Body::App(ref mut f, ref mut x) => f.apply_by(id, val) | x.apply_by(id, val),
        };
        if changed {
            self.closed &= val.closed;
        }
        changed
    }

    pub fn fix_captures(&mut self, rhs: &Self) {
        // closed terms don't have any free variable, so vars /\ free is always {}
        if rhs.closed {
            return;
        }
        let frees_val = rhs.free_variables();
        let vars = self.bounded_variables();
        // if there's no free variable capturing (used vars on lhs /\ free vars on rhs), we just apply on the abstraction body
        let captures: Vec<_> = frees_val.intersection(&vars).collect(); // TODO: Use vec
        if !captures.is_empty() {
            self.redex_by_alpha(&mut captures.into_iter().map(|&i| (i, i)).collect());
        }
    }

    pub fn beta_redex(&mut self) {
        // self.update_closed();
        while self.beta_redex_step() {}
    }

    pub fn beta_redex_step(&mut self) -> bool {
        // assert!(!self.closed || self.free_variables().is_empty());
        match Rc::make_mut(&mut self.body) {
            Body::Id(..) => false,
            Body::App(ref mut f, ref mut x) => {
                if matches!(*f.body, Body::Abs(..)) {
                    f.fix_captures(x);
                    let (id, l) = f.as_mut_abs().unwrap();
                    l.apply_by(*id, x);
                    *self = l.clone();
                    true
                } else {
                    f.beta_redex_step() || x.beta_redex_step()
                }
            }
            Body::Abs(..) => {
                self.eta_redex_step() || self.as_mut_abs().unwrap().1.beta_redex_step()
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
        match *self.body {
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
        if let Body::Abs(v, ref mut app) = Rc::make_mut(&mut self.body) {
            if let Body::App(ref mut lhs, ref mut rhs) = Rc::make_mut(&mut app.body) {
                if rhs.body.as_ref() == &Body::Id(*v) && !lhs.contains(&Body::Id(*v)) {
                    *self = lhs.clone();
                    return true;
                }
            }
        }
        false
    }

    /// returns if, at any depth, starting from the outmost expression, there's the passed
    /// expression.
    pub fn contains(&self, what: &Body) -> bool {
        match self.body.as_ref() {
            Body::Id(..) => *self.body == *what,
            Body::App(lhs, rhs) => {
                self.body == lhs.body
                    || self.body == rhs.body
                    || lhs.contains(what)
                    || rhs.contains(what)
            }
            Body::Abs(_, l) => *self.body == *what || l.contains(what),
        }
    }

    pub fn try_from_str<T: AsRef<str>>(s: T) -> Result<Self, lrp::Error<parser::Sym>> {
        let lex = parser::try_lexer(s.as_ref())?;
        parser::parse(lex)
    }

    pub fn debrejin_reduced(mut self) -> Self {
        self.debrejin_redex();
        self
    }

    /// DeBrejin alpha-reduce the expression, renaming variables according to the depth level, starting
    /// from 0 and skipping when there's a free variable with same id
    /// debrejin_redex(^x.(x b) ^b.(b b)) = ^a.(a b) ^c.(c c)
    /// Sounds like a worthless method, but actually, it turns every expression into a same alpha
    /// expression. I.e:
    /// (^x.(x b) ^b.(b b)) (^y.(y b) ^l.(l l)) == false, and
    /// alpha_eq (^x.(x b) ^b.(b b)) (^y.(y b) ^l.(l l)) == true, but
    /// eq alpha(^x.(x b) ^b.(b b)) alpha(^y.(y b) ^l.(l l)) == false, while
    /// eq debrejin(^x.(x b) ^b.(b b)) debrejin(^y.(y b) ^l.(l l)) == true
    pub fn debrejin_redex(&mut self) {
        let mut binds = self.free_variables().iter().map(|&i| (i, i)).collect();
        self.redex_by_debrejin(&mut binds, 0);
    }

    pub fn redex_by_debrejin(&mut self, binds: &mut HashMap<VarId, VarId>, lvl: usize) {
        match Rc::make_mut(&mut self.body) {
            Body::Id(ref mut id) => {
                if binds.contains_key(id) {
                    *id = binds[id]
                }
            }
            Body::App(ref mut lhs, ref mut rhs) => {
                lhs.redex_by_debrejin(binds, lvl + 1);
                rhs.redex_by_debrejin(binds, lvl + 1);
            }
            Body::Abs(ref mut v, ref mut l) => {
                let lvl = Self::get_next_free(lvl, binds);
                let old_v = *v;
                *v = lvl;
                let old = binds.insert(old_v, lvl);
                l.redex_by_debrejin(binds, lvl + 1);
                if let Some(old) = old {
                    *binds.get_mut(&old_v).unwrap() = old;
                } else {
                    binds.remove(&lvl);
                }
            }
        }
    }

    pub fn get_next_free(start: VarId, binds: &HashMap<VarId, VarId>) -> VarId {
        for k in start.. {
            if !binds.contains_key(&k) {
                return k;
            }
        }
        unreachable!("how the 2^64 - 1 possible var ids was used, my man?");
    }

    /// Checks if an expression is debrejin alpha compatible. Notice that, the set of `is_debrejin`
    /// is bigger than the amount of expressions from debrejin_redex contradomain, as it treats
    /// ^a.(a b) ^a.(^b.(b)) as debrejin valid. While the true debrejin form is ^a.(a b) ^a.(^c.(c))
    pub fn is_debrejin(&self) -> bool {
        self.check_is_debrejin(0)
    }

    pub fn check_is_debrejin(&self, lvl: usize) -> bool {
        match self.body.as_ref() {
            Body::Id(..) => true,
            Body::App(lhs, rhs) => lhs.check_is_debrejin(lvl + 1) && rhs.check_is_debrejin(lvl + 1),
            Body::Abs(v, l) => *v == lvl && l.check_is_debrejin(lvl + 1),
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum Body {
    /* identity */ Id(VarId),
    /* application */ App(Term, /* ( */ Term /* ) */),
    /* abstraction */ Abs(VarId, Term),
}

impl Body {
    /// returns all free variables, including the ones binded on one application
    /// FV(^x.(x a x) b) = { a, b }
    /// FV(^x.(a) ^a.(a)) = { a }
    /// FV(a) = { a }
    #[must_use]
    pub fn free_variables(&self) -> HashSet<VarId> {
        let (mut binds, mut frees) = (HashSet::default(), HashSet::default());
        self.get_free_variables(&mut binds, &mut frees);
        frees
    }

    pub fn get_free_variables(&self, binds: &mut HashSet<VarId>, frees: &mut HashSet<VarId>) {
        match self {
            Self::Id(id) => {
                if !binds.contains(id) {
                    frees.insert(*id);
                }
            }
            Self::App(lhs, rhs) => {
                if !lhs.closed {
                    lhs.body.get_free_variables(binds, frees);
                }
                if !rhs.closed {
                    rhs.body.get_free_variables(binds, frees);
                }
            }
            Self::Abs(v, l) => {
                if !l.closed {
                    let recent = binds.insert(*v);
                    l.body.get_free_variables(binds, frees);
                    if recent {
                        binds.remove(v);
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn bounded_variables(&self) -> HashSet<VarId> {
        let mut binds = HashSet::default();
        self.get_bounded_variables(&mut binds);
        binds
    }

    fn get_bounded_variables(&self, binds: &mut HashSet<VarId>) {
        match self {
            Self::Id(..) => (),
            Self::App(lhs, rhs) => {
                lhs.body.get_bounded_variables(binds);
                rhs.body.get_bounded_variables(binds);
            }
            Self::Abs(v, l) => {
                binds.insert(*v);
                l.body.get_bounded_variables(binds);
            }
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

#[cfg(test)]
pub mod tests {
    use crate::Term;

    #[test]
    pub fn valid_syntax() {
        const SCRIPTS: &[&str] = &[
            "^x.(a)",
            "\\x.(x (a c))",
            "deadbeef",
            "λl.(l l)",
            "(x (x) a)",
            "\\i->(a c)",
        ];
        SCRIPTS
            .iter()
            .for_each(|s| assert!(Term::try_from_str(s).is_ok()))
    }

    #[test]
    pub fn invalid_syntax() {
        const SCRIPTS: &[&str] = &["^x.()", "(x x) a)", "^x(a)", "DEADBEEF", "\\\\x.(a)"];
        SCRIPTS
            .iter()
            .for_each(|s| assert!(Term::try_from_str(s).is_err()))
    }
}
