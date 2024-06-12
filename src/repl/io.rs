use crate::cci::scope::Scope;

use super::CmdEntry;

pub fn reload(e: CmdEntry) {
    e.repl.runner.scope = Scope::default();
    let mut v = Vec::new();
    std::mem::swap(&mut v, &mut e.repl.runner.loaded_files);
    v.into_iter().for_each(|fp| match e.repl.runner.load(&fp) {
        Ok(()) => (),
        Err(e) => eprintln!("error reloading environment: {e:?}"),
    });
}
