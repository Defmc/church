use church::{Body, Term};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::atomic::AtomicUsize};

use crate::{Ast, Error, Result};

#[derive(Default, Clone)]
pub struct Compiler {
    pub defs: HashMap<String, Term>,
    pub aliases: HashMap<Term, String>,
}

impl Compiler {
    pub fn dump(&mut self, t: &Ast) -> Result<Term> {
        self.dump_with(&mut HashMap::new(), t)
    }

    fn dump_with(&mut self, ctx: &mut HashMap<String, usize>, t: &Ast) -> Result<Term> {
        match t {
            Ast::Var(v) => self.get_var_def(ctx, v),
            Ast::App(m, n) => {
                let m = self.dump_with(ctx, m)?;
                let n = self.dump_with(ctx, n)?;
                let b = Body::App(m, n);
                Ok(Term::from(b))
            }
            Ast::Abs(v, m) => {
                let v_alias = Self::get_new_ident();
                let old = ctx.insert(v.clone(), v_alias);
                let m = self.dump_with(ctx, m)?;
                if let Some(old) = old {
                    *ctx.get_mut(v).unwrap() = old;
                }
                let b = Body::Abs(v_alias, m);
                Ok(Term::from(b))
            }
            Ast::Let(defs, m) => self.dump_let(ctx, defs, m),
            _ => todo!(),
        }
    }

    fn dump_let(
        &mut self,
        ctx: &mut HashMap<String, usize>,
        defs: &[Ast],
        m: &Ast,
    ) -> Result<Term> {
        let mut olds = Vec::new();
        for component in defs {
            if let Ast::Assign(id, def) = component {
                if let Some(old_bind) = self.defs.remove(id) {
                    let old_alias = self.aliases.remove(&old_bind);
                    olds.push((Some(old_bind), old_alias));
                } else {
                    olds.push((None, None));
                }
                let dump = self.dump(def)?;
                self.insert(id.clone(), dump)?;
            } else {
                unreachable!()
            }
        }
        let term = self.dump_with(ctx, m)?;
        for (component, (old_bind, old_alias)) in defs.iter().zip(olds) {
            if let Ast::Assign(id, _) = component {
                if let Some(bind) = old_bind {
                    *self.aliases.get_mut(&bind).unwrap() = old_alias.unwrap();
                    *self.defs.get_mut(id).unwrap() = bind;
                } else {
                    self.defs.remove(id);
                }
            } else {
                unreachable!()
            }
        }
        Ok(term)
    }

    fn get_var_def(&self, ctx: &mut HashMap<String, usize>, v: &str) -> Result<Term> {
        ctx.get(v)
            .map_or_else(
                || self.defs.get(v).cloned(),
                |v| Some(Term::from(Body::Var(*v))),
            )
            .or_else(|| Self::into_free_var(v))
            .ok_or_else(|| Error::DefNotFound(v.to_string()))
    }

    //#[cfg(not(feature = "aliased-vars"))]
    //fn get_idx(f: impl Iterator<Item = char>) -> Option<usize> {
    //    todo!()
    //}

    //#[cfg(feature = "aliased-vars")]
    fn get_idx<I>(s: I) -> Option<usize>
    where
        I: Iterator<Item = char> + DoubleEndedIterator,
    {
        const ALIASES: &[char] = &[
            'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'μ', 'ν', 'ξ', 'ο', 'π', 'ρ', 'σ',
            'ς', 'τ', 'υ', 'φ', 'χ', 'ψ', 'ω', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
            'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0',
            '1', '2', '3', '4', '5', '6', '7', '8', '9',
        ];
        static LAZY_MAP: Lazy<HashMap<char, usize>> = Lazy::new(|| {
            ALIASES
                .iter()
                .copied()
                .enumerate()
                .map(|(i, c)| (c, i))
                .collect()
        });

        let mut counter = 0;
        for c in s.rev() {
            let n = LAZY_MAP.get(&c)?;
            counter = counter * 10 + n;
        }
        Some(counter)
    }

    fn into_free_var(s: &str) -> Option<Term> {
        let t = Body::Var(Self::get_idx(s.chars())?);
        Some(t.into())
    }

    fn get_new_ident() -> usize {
        static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn insert(&mut self, name: String, def: Term) -> Result<()> {
        if self.defs.contains_key(&name) {
            let t = self.defs[&name].clone();
            Err(Error::AlreadyDefined(name, t))
        } else {
            self.aliases
                .insert(def.coerce(Term::unique_alpha_redex), name.clone());
            self.defs.insert(name, def);

            Ok(())
        }
    }

    pub fn pretty_show(&self, t: &Term) -> String {
        if let Some(s) = self.aliases.get(&t.coerce(Term::unique_alpha_redex)) {
            s.clone()
        } else {
            match t.body.as_ref() {
                Body::Var(v) => Self::get_alias(*v),
                Body::App(m, n) => format!("{} {}", self.pretty_show(m), self.pretty_show(n)),
                Body::Abs(v, m) => {
                    format!("λ{} {}", Self::get_alias(*v), self.pretty_show(m))
                }
            }
        }
    }

    pub fn get_alias(var: usize) -> String {
        const ALIASES: &[char] = &[
            'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'μ', 'ν', 'ξ', 'ο', 'π', 'ρ', 'σ',
            'ς', 'τ', 'υ', 'φ', 'χ', 'ψ', 'ω', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
            'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0',
            '1', '2', '3', '4', '5', '6', '7', '8', '9',
        ];
        let mut counter = var;
        let mut s = String::with_capacity(var / ALIASES.len());
        loop {
            let idx = counter % ALIASES.len();
            s.push(ALIASES[idx]);
            counter /= ALIASES.len();
            if counter == 0 {
                return s;
            }
        }
    }
}
