use crate::{Body, Term, VarId};
use core::fmt;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::num::NonZeroUsize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum UnprocessedBody {
    Var(String),
    Abs(String, Box<Self>),
    App(Box<Self>, Box<Self>),
}

impl UnprocessedBody {
    pub fn dump(&self) -> Term {
        let mut set = HashSet::default();
        self.get_used_vars(&mut set);

        let renames: HashMap<String, VarId> = set
            .iter()
            .cloned()
            .zip(Self::get_free_names(&set))
            .collect();
        self.dump_with(&renames)
    }

    pub fn dump_with(&self, map: &HashMap<String, VarId>) -> Term {
        match self {
            Self::Var(s) => Term::new(Body::Id(map[s])),
            Self::App(lhs, rhs) => Term::new(Body::App(lhs.dump_with(map), rhs.dump_with(map))),
            Self::Abs(arg, fun) => Term::new(Body::Abs(map[arg], fun.dump_with(map))),
        }
    }

    pub fn get_used_vars(&self, set: &mut HashSet<String>) {
        match self {
            Self::Var(s) => {
                if !set.contains(s) {
                    set.insert(s.clone());
                }
            }
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

    pub fn get_free_names(used_names: &HashSet<String>) -> impl Iterator<Item = VarId> + '_ {
        (0..).filter(|&i| !used_names.contains(&crate::id_to_str(i)))
    }

    #[must_use]
    pub fn len(&self) -> NonZeroUsize {
        match *self {
            Self::Var(..) => 1.try_into().unwrap(),
            Self::App(ref f, ref x) => f.len().saturating_add(x.len().into()),
            Self::Abs(_, ref b) => b.len().saturating_add(1),
        }
    }

    pub fn try_from_str<T: AsRef<str>>(s: T) -> Result<Self, lrp::Error<super::Sym>> {
        let lex = super::try_lexer(s.as_ref())?;
        super::parse(lex)
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
