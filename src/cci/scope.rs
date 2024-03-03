use church::Term;
use rustc_hash::FxHashMap as HashMap;

use super::ubody::{Dumper, UnprocessedBody};

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

    pub fn include_from_ubody(&mut self, def: &str, imp: &UnprocessedBody) {
        let mut dumper = Dumper::new(self);
        let t = dumper.rec_dump(def, imp);
        self.include(def, t);
    }

    pub fn get_like(&self, t: &Term) -> Option<&str> {
        self.alias
            .get(&t.clone().debrejin_reduced())
            .map(|s| s.as_str())
    }

    pub fn delta_redex(&self, u: &UnprocessedBody) -> Term {
        let mut dumper = Dumper::new(self);
        dumper.dump(u)
    }

    pub fn print(&self, t: &Term) {
        todo!()
    }
}
