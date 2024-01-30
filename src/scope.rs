use rustc_hash::FxHashMap as HashMap;
use std::str::FromStr;

use aho_corasick::AhoCorasick;

use crate::{parser, Term};

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
                    if self.defs[*iv] != v {
                        panic!(
                            "shadowing {k:?}: the old value {:?} is different from the new {v:?}",
                            self.defs[*iv]
                        );
                    }
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
}

impl FromStr for Scope {
    type Err = lrp::Error<parser::Sym>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut defs = HashMap::default();
        for l in s.lines() {
            let end = l.find(|c| c == '#').unwrap_or(l.len());
            let l = &l[..end];
            if let Some(equal_pos) = l.find(|c| c == '=') {
                let bind = &l[..equal_pos].trim();
                let imp = &l[equal_pos + 1..].trim();
                if let Some(shadow) = defs.insert(bind.to_string(), imp.to_string()) {
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
