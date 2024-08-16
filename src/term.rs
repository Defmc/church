use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Write},
};

/// An lambda body's expression
/// x is a variable.
/// M N is an application where M and N are lambda terms.
/// λx.M is an abstraction (function) where x is the introduced variable and M the function's body.
#[derive(Eq, Clone, PartialEq, Debug, Hash)]
pub enum Body {
    Var(usize),
    App(Term, Term),
    Abs(usize, Term),
}

#[derive(Eq, Clone, PartialEq, Debug, Hash)]
pub struct Term {
    pub body: Box<Body>,
}

impl Term {
    pub fn coerce(&self, f: impl Fn(&mut Self)) -> Self {
        let mut clone = self.clone();
        f(&mut clone);
        clone
    }

    pub fn free_vars(&self) -> HashSet<usize> {
        let (mut closeds, mut frees) = (HashSet::new(), HashSet::new());
        self.free_vars_from(&mut closeds, &mut frees);
        frees
    }

    fn free_vars_from(&self, closeds: &mut HashSet<usize>, frees: &mut HashSet<usize>) {
        match self.body.as_ref() {
            Body::Var(v) => {
                if !closeds.contains(v) {
                    frees.insert(*v);
                }
            }
            Body::App(m, n) => {
                m.free_vars_from(closeds, frees);
                n.free_vars_from(closeds, frees);
            }
            Body::Abs(v, m) => {
                let new = closeds.insert(*v);
                m.free_vars_from(closeds, frees);
                if new {
                    closeds.remove(v);
                }
            }
        }
    }

    pub fn unique_alpha_redex(&mut self) {
        let frees = self.free_vars();
        self.unique_alpha_replace(&mut 0, &mut HashMap::new(), &frees);
    }

    fn unique_alpha_replace(
        &mut self,
        next: &mut usize,
        replaces: &mut HashMap<usize, usize>,
        frees: &HashSet<usize>,
    ) {
        match self.body.as_mut() {
            Body::Var(v) => {
                if let Some(nv) = replaces.get(v) {
                    *v = *nv;
                } else {
                    assert!(frees.contains(v));
                }
            }
            Body::App(m, n) => {
                m.unique_alpha_replace(next, replaces, frees);
                n.unique_alpha_replace(next, replaces, frees);
            }
            Body::Abs(v, m) => {
                // SAFETY: As a unique alpha reducer, it just increase after each iteration,
                // so it's impossible to have a already used replacement index while reducing,
                // just the `frees` can appear.
                let nv = (*next..).find(|n| !frees.contains(n)).unwrap();
                replaces.insert(*v, nv);
                *v = nv;
                *next = nv + 1;
                m.unique_alpha_replace(next, replaces, frees);
            }
        }
    }

    /// Applicates the normal β-reduction on the term,
    /// where the leftmost outmost term is evaluated first,
    /// returns a `bool` indicating if it's on its normal form (a.f.k irreducible)
    /// I.e, it computes the function before each argument.
    pub fn normal_beta_redex_step(&mut self) -> bool {
        match self.body.as_mut() {
            Body::App(m, n) => {
                if m.normal_beta_redex_step() {
                    if let Body::Abs(v, b) = m.body.as_mut() {
                        b.apply(*v, n);
                        *self = b.clone(); // FIXME: Shouldn't clone
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            Body::Abs(_, m) => m.normal_beta_redex_step(),
            Body::Var(..) => true,
        }
    }

    /// Applicates the call-by-value β-reduction on the term,
    /// where the innermost right term is evaluated first,
    /// returns a `bool` indicating if it's on its normal form (a.f.k irreducible)
    /// I.e, it computes the arguments before function.
    pub fn cbv_beta_redex_step(&mut self) -> bool {
        match self.body.as_mut() {
            Body::App(m, n) => {
                if n.cbv_beta_redex_step() {
                    if let Body::Abs(v, b) = m.body.as_mut() {
                        b.apply(*v, n);
                        *self = b.clone(); // FIXME: Shouldn't clone
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            Body::Abs(_, m) => m.cbv_beta_redex_step(),
            Body::Var(..) => true,
        }
    }

    pub fn apply(&mut self, var: usize, val: &Self) {
        match self.body.as_mut() {
            Body::Var(v) => {
                if *v == var {
                    *self = val.clone();
                }
            }
            Body::App(m, n) => {
                m.apply(var, val);
                n.apply(var, val);
            }
            Body::Abs(v, m) => {
                if *v != var {
                    m.apply(var, val)
                }
            }
        }
    }
}

impl From<Body> for Term {
    fn from(value: Body) -> Self {
        Self {
            body: Box::new(value),
        }
    }
}

#[cfg(not(feature = "aliased-vars"))]
pub fn write_alias(idx: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(&idx, f)
}

#[cfg(feature = "aliased-vars")]
pub fn write_alias(var: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    const ALIASES: &[char] = &[
        'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'μ', 'ν', 'ξ', 'ο', 'π', 'ρ', 'σ', 'ς',
        'τ', 'υ', 'φ', 'χ', 'ψ', 'ω', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
        'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3',
        '4', '5', '6', '7', '8', '9',
    ];
    let mut counter = var;
    loop {
        let idx = counter % ALIASES.len();
        fmt::Display::fmt(&ALIASES[idx], f)?;
        counter /= ALIASES.len();
        if counter == 0 {
            return fmt::Result::Ok(());
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.body.as_ref() {
            Body::Var(v) => write_alias(*v, f),
            Body::App(m, n) => write!(
                f,
                "{} {n}",
                if !matches!(m.body.as_ref(), Body::Var(..)) {
                    format!("({m})")
                } else {
                    m.to_string()
                }
            ),
            Body::Abs(v, m) => {
                f.write_str("λ")?;
                write_alias(*v, f)?;
                f.write_char('.')?;
                write!(f, "{m}")
            }
        }
    }
}
