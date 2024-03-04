use rustc_hash::FxHashMap as HashMap;
use std::{iter::Peekable, str::FromStr};

use aho_corasick::AhoCorasick;

use crate::{id_to_str, parser, Term, VarId};

pub struct TabulatedLines<'a, I: Iterator<Item = &'a str>>(pub Peekable<I>);

impl<'a, I: Iterator<Item = &'a str>> Iterator for TabulatedLines<'a, I> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.next()?;
        let mut s = next.to_owned();
        while let Some(p) = self.0.peek() {
            if p.starts_with(' ') || p.starts_with('\t') {
                s.push_str(self.0.next().unwrap());
            } else {
                break;
            }
        }
        Some(s)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Scope {
    // alias: definition
    pub aliases: Vec<String>,
    pub defs: Vec<String>,
    pub indexes: HashMap<String, usize>,
    // post-processed definition: alias
    pub cached_defs: HashMap<String, String>,
    pub need_update: bool,
}

impl Scope {
    pub fn delta_redex(&mut self, b: &str) -> (String, bool) {
        self.update();
        self.redex_by_delta(b)
    }

    pub fn redex_by_delta(&self, b: &str) -> (String, bool) {
        let ac = AhoCorasick::builder()
            .match_kind(aho_corasick::MatchKind::LeftmostLongest)
            .build(&self.aliases)
            .unwrap();
        let result = ac.replace_all(b, &self.defs);
        let changed = result != b;
        (result, changed)
    }

    pub fn internal_delta(&mut self) {
        let mut changed = true;
        let need = self.need_update;
        self.need_update = false;
        while changed {
            changed = false;
            for i in 0..self.defs.len() {
                let (result, diff) = self.redex_by_delta(&self.defs[i]);
                if diff {
                    self.defs[i] = result;
                    changed = true;
                }
            }
        }
        self.need_update = need;
    }

    pub fn update(&mut self) {
        if !self.need_update {
            return;
        }
        self.internal_delta();
        self.cache_defs();
        self.indexes = self
            .aliases
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, a)| (a, i))
            .collect();
        self.index();
        self.need_update = false;
    }

    pub fn index(&mut self) {
        self.indexes
            .reserve(self.aliases.len() - self.indexes.len());
        self.indexes.clear();
        for (i, (k, v)) in self
            .aliases
            .clone()
            .into_iter()
            .zip(self.defs.clone().into_iter())
            .enumerate()
        {
            self.indexes
                .entry(k.clone())
                .and_modify(|iv| {
                    //     if self.defs[*iv] != v {
                    //         panic!(
                    //             "shadowing {k:?}: the old value {:?} is different from the new {v:?}",
                    //             self.defs[*iv]
                    //         );
                    //      }
                    self.defs[*iv] = v;
                })
                .or_insert(i);
        }
    }

    pub fn cache_defs(&mut self) {
        self.cached_defs.clear();
        for (k, v) in self.aliases.iter().zip(self.defs.iter()) {
            match Term::try_from_str(v) {
                Ok(l) => {
                    self.cached_defs
                        .insert(l.debrejin_reduced().to_string(), k.clone());
                }
                Err(e) => eprintln!("error: {e:?}"),
            }
        }
    }

    pub fn extend(&mut self, rhs: Self) {
        self.defs.extend(rhs.defs);
        self.aliases.extend(rhs.aliases);
        self.need_update = true;
    }

    pub fn get_from_alpha_key(&self, key: &Term) -> Option<&str> {
        self.cached_defs
            .get(&key.clone().debrejin_reduced().to_string())
            .map(|s| s.as_str())
    }

    pub fn solve_recursion(def: &str, imp: &str) -> Option<String> {
        if imp.contains(def) {
            let free_name = Self::get_free_name(imp);
            let aho = AhoCorasick::builder()
                .match_kind(aho_corasick::MatchKind::LeftmostLongest)
                .build([def])
                .unwrap();
            let s = aho.replace_all(imp, &[id_to_str(free_name)]);
            Some(format!(
                "(^f.(^x.(f (x x)) ^x.(f (x x))) ^{}.({s}))",
                id_to_str(free_name)
            ))
        } else {
            None
        }
    }

    pub fn get_free_name(imp: &str) -> VarId {
        let mut free = 0;
        loop {
            let aho = AhoCorasick::builder()
                .match_kind(aho_corasick::MatchKind::LeftmostLongest)
                .build([id_to_str(free)])
                .unwrap();
            if !aho.is_match(imp) {
                return free;
            }
            free += 1;
        }
    }
}

impl FromStr for Scope {
    type Err = lrp::Error<parser::Sym>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut defs = HashMap::default();
        for l in TabulatedLines(s.lines().peekable()) {
            let end = l.find(|c| c == '#').unwrap_or(l.len());
            let l = &l[..end];
            if let Some(equal_pos) = l.find(|c| c == '=') {
                let bind = &l[..equal_pos].trim();
                let imp = &l[equal_pos + 1..].trim();
                let imp = Self::solve_recursion(bind, imp).unwrap_or_else(|| format!("({})", imp));
                if let Some(shadow) = defs.insert(bind.to_string(), imp) {
                    panic!("shadowing {bind}, already defined as {shadow}");
                }
            }
        }
        let s = Scope {
            aliases: defs.keys().cloned().collect(),
            defs: defs.values().cloned().collect(),
            cached_defs: HashMap::default(),
            indexes: HashMap::default(),
            need_update: true,
        };
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::Term;

    use super::Scope;
    use std::str::FromStr;

    #[test]
    fn constants_replacement() {
        let mut s = Scope::from_str(
            "
C = ^x.(x x)
A = a a
        ",
        )
        .unwrap();
        let lhs = Term::from_str(&s.delta_redex("C A").0)
            .unwrap()
            .beta_reduced();
        let rhs = Term::from_str(&s.delta_redex("C (A)").0)
            .unwrap()
            .beta_reduced();
        assert!(lhs.alpha_eq(&rhs), "{lhs} != {rhs}");
    }
}
