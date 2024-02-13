use std::{collections::HashSet, fs::read_to_string};

use super::CmdEntry;
use church::scope::{Scope, TabulatedLines};

pub fn run(e: CmdEntry) {
    let input = e.inputs[0].into();
    match read_to_string(&input) {
        Ok(s) => {
            TabulatedLines(s.lines().peekable()).for_each(|l| e.repl.parse(&l));
            e.repl.loaded_files.insert(input);
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
}
pub fn rerun(e: CmdEntry) {
    e.repl.scope = Scope::default();
    let loaded = e.repl.loaded_files.clone();
    e.repl.loaded_files.clear();
    loaded.into_iter().for_each(|f| {
        run(CmdEntry {
            inputs: vec![&f.to_string_lossy()],
            flags: HashSet::default(),
            repl: e.repl,
        })
    });
}

pub fn load(e: CmdEntry) {
    let input = e.inputs[0].into();
    if e.repl.loaded_files.contains(&input) {
        eprintln!("warn: already loaded {input:?}");
        return;
    }
    match read_to_string(&input) {
        Ok(s) => {
            TabulatedLines(s.lines().peekable())
                // fix: there're expressions # like = this
                .filter(|l| l.starts_with(':') || l.contains('='))
                .for_each(|l| e.repl.parse(&l));
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
