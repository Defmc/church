use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::Term;

#[derive(Default, Debug, Clone)]
pub struct Scope {
    pub definitions: HashMap<String, Term>,
}

impl Scope {
    pub fn include(&mut self, def: String, t: Term) {
        self.definitions.insert(def, t);
    }
}
