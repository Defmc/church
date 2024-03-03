use rustc_hash::FxHashMap as HashMap;

use crate::Term;

#[derive(Default, Debug, Clone)]
pub struct Scope {
    pub definitions: HashMap<String, Term>,
}

impl Scope {}
