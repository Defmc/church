use rustc_hash::FxHashMap as HashMap;

use crate::Term;

#[derive(Default, Debug, Clone)]
pub struct Scope {
    pub definitions: HashMap<String, Term>,
    pub alias: HashMap<Term, String>,
}

impl Scope {
    pub fn include(&mut self, def: &str, t: Term) {
        self.definitions.insert(def.to_string(), t.clone());
        self.alias.insert(t.debrejin_reduced(), def.to_string());
    }

    pub fn get_like(&self, t: &Term) -> Option<&str> {
        self.alias
            .get(&t.clone().debrejin_reduced())
            .map(|s| s.as_str())
    }
}
