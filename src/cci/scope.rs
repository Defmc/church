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
        let t = t.debrejin_reduced();
        self.definitions.insert(def.to_string(), t.clone());
        self.alias.insert(t, def.to_string());
    }

    pub fn include_from_ubody(&mut self, def: &str, imp: &UnprocessedBody) -> bool {
        let mut dumper = Dumper::new(self);
        match dumper.rec_dump(def, imp) {
            Some(t) => {
                self.include(def, t);
                true
            }
            None => false,
        }
    }

    pub fn get_like(&self, t: &Term) -> Option<&str> {
        // println!(
        //     "found like: {:?}",
        //     self.alias
        //         .get(&t.clone().debrejin_reduced())
        //         .map(String::as_str)
        // );
        self.alias
            .get(&t.clone().debrejin_reduced())
            .map(String::as_str)
    }

    pub fn delta_redex(&self, u: &UnprocessedBody) -> Option<Term> {
        let mut dumper = Dumper::new(self);
        dumper.dump(u)
    }
}
