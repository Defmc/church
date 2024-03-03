use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::Term;

#[derive(Default, Debug, Clone)]
pub struct Scope {
    pub definitions: HashMap<String, Term>,
    pub reserved_vars: HashSet<String>,
}

impl Scope {
    pub fn include(&mut self, def: String, t: Term) {
        self.reserved_vars
            .extend(t.free_variables().into_iter().map(crate::id_to_str));
        self.definitions.insert(def, t);
    }
}
