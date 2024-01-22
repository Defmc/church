use std::{collections::HashMap, str::FromStr};

use crate::{parser, Body};

#[derive(Debug, Clone, Default)]
pub struct Scope {
    // alias: definition
    pub defs: HashMap<String, String>,
    // post-processed definition: alias
    pub cached_defs: HashMap<String, String>,
}

impl Scope {
    pub fn delta_redex(&self, b: &mut String) -> bool {
        let mut changed = false;
        for (k, v) in self.defs.iter() {
            if b.contains(k) {
                *b = b.replace(k, v);
                changed = true;
            }
        }
        changed
    }

    pub fn internal_delta(&mut self) {
        let mut changed = true;
        let mut exprs: Vec<_> = self
            .defs
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        while changed {
            changed = false;
            for (k, expr) in exprs.iter_mut() {
                if self.delta_redex(expr) {
                    *self.defs.get_mut(k).unwrap() = expr.clone();
                }
            }
        }
    }

    pub fn update(&mut self) {
        self.internal_delta();
        self.cache_defs();
    }

    pub fn cache_defs(&mut self) {
        self.cached_defs.clear();
        for (k, v) in self.defs.iter() {
            if let Ok(l) = Body::from_str(&v) {
                self.cached_defs
                    .insert(l.alpha_reduced().to_string(), k.clone());
            }
        }
    }

    pub fn extend(&mut self, rhs: Self) {
        self.defs.extend(rhs.defs);
        self.update();
    }

    pub fn get_from_alpha_key(&self, key: &Body) -> Option<&str> {
        self.cached_defs.get(&key.to_string()).map(|s| s.as_str())
    }
}

impl FromStr for Scope {
    type Err = lrp::Error<parser::Sym>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut defs = HashMap::new();
        for l in s.lines() {
            let end = l.find(|c| c == '#').unwrap_or(l.len());
            let l = &l[..end];
            if let Some(equal_pos) = l.find(|c| c == '=') {
                let bind = &l[..equal_pos].trim();
                assert!(
                    bind.chars().take(1).next().unwrap().is_uppercase(),
                    "{bind} doesn't start with a uppercase letter"
                );
                assert!(
                    bind.chars().all(|c| c.is_alphanumeric() || c == '_'),
                    "{bind} doesn't start with a uppercase letter"
                );
                let imp = &l[equal_pos + 1..].trim();
                if let Some(shadow) = defs.insert(bind.to_string(), imp.to_string()) {
                    panic!("shadowing {bind}, already defined as {shadow}");
                }
            }
        }
        let mut s = Scope {
            defs,
            cached_defs: HashMap::new(),
        };
        s.update();
        Ok(s)
    }
}
