use std::{collections::HashMap, str::FromStr};

use crate::parser;

#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub defs: HashMap<String, String>,
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
    pub fn extend(&mut self, rhs: Self) {
        self.defs.extend(rhs.defs);
        self.internal_delta()
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
        let mut s = Scope { defs };
        s.internal_delta();
        Ok(s)
    }
}
