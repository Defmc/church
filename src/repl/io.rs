use std::{collections::HashSet, fs::read_to_string};

use super::CmdEntry;
use church::scope::Scope;

pub fn load(e: CmdEntry) {
    let input = e.inputs[0].into();
    if e.repl.loaded_files.contains(&input) {
        eprintln!("warn: already loaded {input:?}");
        return;
    }
    match read_to_string(&input) {
        Ok(s) => {
            s.lines().for_each(|l| e.repl.parse(l));
            e.repl.loaded_files.insert(input);
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
    if e.flags.contains(&"s") {
        e.repl.scope.update();
    }
}

pub fn reload(e: CmdEntry) {
    e.repl.scope = Scope::default();
    let loaded = e.repl.loaded_files.clone();
    e.repl.loaded_files.clear();
    loaded.into_iter().for_each(|f| {
        load(CmdEntry {
            inputs: vec![&f.to_string_lossy()],
            flags: HashSet::default(),
            repl: e.repl,
        })
    });

    if e.flags.contains(&"s") {
        e.repl.scope.update();
    }
}

pub fn prepare(e: CmdEntry) {
    e.repl.scope.update();
}
