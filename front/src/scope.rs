use std::{collections::HashMap, str::FromStr, sync::atomic::AtomicUsize};

use church::{Body, Term};

use crate::{parser::ParserBodyError, UBody, UTerm};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Definition for {0} wasn't found")]
    DefNotFound(String),

    #[error("{0}")]
    ParserError(ParserBodyError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct Scope {
    pub defs: HashMap<String, Term>,
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
}
