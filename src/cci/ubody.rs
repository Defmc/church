use crate::{id_to_str, Body, Term, VarId};
use core::fmt;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::num::NonZeroUsize;

use super::scope::Scope;

pub struct Dumper<'a> {
    scope: &'a Scope,
    renames: HashMap<String, VarId>,
    last_var_id: VarId,
}

impl<'a> Dumper<'a> {
    pub fn new(scope: &'a Scope) -> Self {
        Self {
            scope,
            renames: HashMap::default(),
            last_var_id: 0,
        }
    }

    pub fn dump(&mut self, expr: &UnprocessedBody) -> Term {
        let mut used_vars = HashSet::default();
        expr.get_used_vars(&mut used_vars);
        self.dump_with(expr)
    }

    pub fn dump_with(&mut self, expr: &UnprocessedBody) -> Term {
        match expr {
            UnprocessedBody::Var(v) => self.handle_vars(v),
            UnprocessedBody::App(lhs, rhs) => {
                self.dump_with(lhs);
                self.dump_with(rhs)
            }
            UnprocessedBody::Abs(v, l) => {
                let (var_id, body) = self.do_binding(v, |s| s.dump_with(l));
                Term::new(Body::Abs(var_id, body))
            }
        }
    }

    pub fn do_binding<T>(&mut self, v: &str, f: impl FnOnce(&mut Self) -> T) -> (usize, T) {
        let old_rename_val = self.renames.get(v).cloned();
        let old_last_var_id = self.last_var_id;
        let var_id = self.get_next_free_name();
        self.renames.insert(v.to_string(), var_id);
        let f_ret = f(self);
        self.last_var_id = old_last_var_id;
        if let Some(rename_val) = old_rename_val {
            *self.renames.get_mut(v).unwrap() = rename_val;
        } else {
            self.renames.remove(v);
        }
        (var_id, f_ret)
    }

    pub fn handle_vars(&mut self, var: &str) -> Term {
        if let Some(id) = self.renames.get(var) {
            Term::new(Body::Id(*id))
        } else if let Some(def) = self.scope.definitions.get(var) {
            def.clone()
        } else {
            panic!("cannot find a definition for {var:?}");
        }
    }

    pub fn get_next_free_name(&mut self) -> VarId {
        let var_id = (self.last_var_id..)
            .filter(|&i| !self.is_var_used(&id_to_str(i)))
            .next()
            .unwrap();
        self.last_var_id = var_id + 1;
        var_id
    }

    pub fn is_var_used(&self, v: &str) -> bool {
        self.renames.contains_key(v) || self.scope.definitions.contains_key(v)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum UnprocessedBody {
    Var(String),
    Abs(String, Box<Self>),
    App(Box<Self>, Box<Self>),
}

impl UnprocessedBody {
    pub fn get_used_vars(&self, set: &mut HashSet<String>) {
        match self {
            Self::Var(_) => {}
            Self::Abs(arg, fun) => {
                if !set.contains(arg) {
                    set.insert(arg.clone());
                }
                fun.get_used_vars(set);
            }
            Self::App(lhs, rhs) => {
                lhs.get_used_vars(set);
                rhs.get_used_vars(set);
            }
        }
    }

    #[must_use]
    pub fn len(&self) -> NonZeroUsize {
        match *self {
            Self::Var(..) => 1.try_into().unwrap(),
            Self::App(ref f, ref x) => f.len().saturating_add(x.len().into()),
            Self::Abs(_, ref b) => b.len().saturating_add(1),
        }
    }
}

impl fmt::Display for UnprocessedBody {
    fn fmt(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(id) => w.write_fmt(format_args!("{id}")),
            Self::App(ref f, ref x) => w.write_fmt(format_args!(
                "{f} {}",
                if usize::from(x.len()) > 1 {
                    format!("({x})")
                } else {
                    format!("{x}")
                }
            )),
            Self::Abs(v, l) => w.write_fmt(format_args!("λ{v}.({l})")),
        }
    }
}
