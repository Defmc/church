use std::{collections::HashMap, str::FromStr, sync::atomic::AtomicUsize};

use church::{Body, Term};

use crate::{parser::ParserBodyError, UBody, UTerm};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Definition for `{0}` wasn't found")]
    DefNotFound(String),

    #[error("{0}")]
    ParserError(ParserBodyError),

    #[error("Variable {0}'ve been already deifned as {1}")]
    AlreadyDefined(String, Term),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct Scope {
    pub defs: HashMap<String, Term>,
    pub aliases: HashMap<Term, String>,
}

impl Scope {
    pub fn dump(&self, t: &UTerm) -> Result<Term> {
        self.dump_with(&mut HashMap::new(), t)
    }

    fn dump_with(&self, ctx: &mut HashMap<String, usize>, t: &UTerm) -> Result<Term> {
        match t.body.as_ref() {
            UBody::Var(v) => self.get_var_def(ctx, v),
            UBody::App(m, n) => {
                let m = self.dump_with(ctx, m)?;
                let n = self.dump_with(ctx, n)?;
                let b = Body::App(m, n);
                Ok(Term::from(b))
            }
            UBody::Abs(v, m) => {
                let v_alias = Self::get_new_ident();
                let old = ctx.insert(v.clone(), v_alias);
                let m = self.dump_with(ctx, m)?;
                if let Some(old) = old {
                    *ctx.get_mut(v).unwrap() = old;
                }
                let b = Body::Abs(v_alias, m);
                Ok(Term::from(b))
            }
        }
    }

    fn get_var_def(&self, ctx: &mut HashMap<String, usize>, v: &str) -> Result<Term> {
        ctx.get(v)
            .map_or_else(
                || self.defs.get(v).cloned(),
                |v| Some(Term::from(Body::Var(*v))),
            )
            .ok_or_else(|| Error::DefNotFound(v.to_string()))
    }

    fn get_new_ident() -> usize {
        static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn into_term(&self, s: &str) -> Result<Term> {
        let t = UTerm::from_str(s).map_err(|e| Error::ParserError(e))?;
        self.dump(&t)
    }

    pub fn insert(&mut self, name: String, def: Term) -> Result<()> {
        if self.defs.contains_key(&name) {
            let t = self.defs[&name].clone();
            Err(Error::AlreadyDefined(name, t))
        } else {
            let _ = self
                .aliases
                .insert(def.coerce(Term::unique_alpha_redex), name.clone())
                .is_none();
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
