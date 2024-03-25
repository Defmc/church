use std::{collections::HashSet, fs::read_to_string};

use crate::cci::scope::Scope;

use super::CmdEntry;

pub fn load(e: CmdEntry) {
    let input = e.inputs[0].into();
    if e.repl.loaded_files.contains(&input) {
        eprintln!("warn: already loaded {input:?}");
        return;
    }
    match read_to_string(&input) {
        Ok(s) => match e.repl.runner.run(&s) {
            Ok(()) => e.repl.loaded_files.insert(input),
            Err(e) => {
                eprintln!("error: {e:?}");
                return;
            }
        },
        Err(e) => {
            eprintln!("error: {e:?}");
            return;
        }
    };
}

pub fn reload(e: CmdEntry) {
    e.repl.runner.scope = Scope::default();
    let loaded = e.repl.loaded_files.clone();
    e.repl.loaded_files.clear();
    loaded.into_iter().for_each(|f| {
        load(CmdEntry {
            inputs: vec![&f.to_string_lossy()],
            flags: HashSet::default(),
            repl: e.repl,
        })
    });
}
