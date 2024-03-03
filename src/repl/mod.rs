use church::{scope::Scope, Body, Term, VarId};
use rustc_hash::FxHashSet as HashSet;
use rustyline::{config::Configurer, error::ReadlineError, DefaultEditor};
use std::{path::PathBuf, str::FromStr, sync::atomic::Ordering};

use crate::{cci::runner::Runner, repl::cmds::CmdEntry};

pub mod cmds;
pub mod env;
pub mod io;
pub mod parser;
pub mod proc;

pub type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct Repl {
    scope: Scope,
    loaded_files: HashSet<PathBuf>,
    prompt: String,
    readable: bool,
    binary_numbers: bool,
    quit: bool,
    rl: DefaultEditor,
    runner: Runner,
}

impl Default for Repl {
    fn default() -> Self {
        let mut rl = DefaultEditor::new().unwrap();
        rl.set_auto_add_history(true);
        rl.set_history_ignore_space(true);
        Repl {
            scope: Scope::default(),
            loaded_files: HashSet::default(),
            readable: true,
            binary_numbers: false,
            quit: false,
            prompt: String::from("λ> "),
            rl,
            runner: Runner::default(),
        }
    }
}

impl Repl {
    pub fn start(&mut self) -> Result {
        if let Err(e) =
            ctrlc::set_handler(|| crate::cci::mode::INTERRUPT.store(true, Ordering::SeqCst))
        {
            eprintln!("error: {e:?}");
        }
        while !self.quit {
            let buf = match self.rl.readline(&self.prompt) {
                Ok(s) => s,
                Err(e) => {
                    return if matches!(e, ReadlineError::Eof) {
                        Ok(())
                    } else {
                        Err(Box::new(e))
                    };
                }
            };
            self.parse(&buf);
        }
        Ok(())
    }

    pub fn parse(&mut self, input: &str) {
        let input = input.trim();
        if input.starts_with(':') {
            let args: Vec<_> = parser::Arg::parse(&input).collect();
            self.handle(&args);
        } else {
            self.runner.run(input);
        }
    }
    pub fn alias(&mut self, input: &str) {
        match Scope::from_str(input) {
            Ok(nscope) => self.scope.extend(nscope),
            Err(e) => eprintln!("error: {e:?}"),
        }
    }

    pub fn handle(&mut self, args: &[&str]) {
        for hs in cmds::COMMANDS.iter() {
            if args[0][1..] == *hs.name {
                let mode = self.runner.mode;
                let entry = CmdEntry {
                    inputs: args
                        .iter()
                        .skip(1)
                        .copied()
                        .filter(|s| !s.starts_with('-'))
                        .collect(),
                    flags: args
                        .iter()
                        .skip(1)
                        .copied()
                        .filter_map(|s| s.strip_prefix('-'))
                        .collect(),
                    repl: self,
                };
                return mode.bench(hs.name, || (hs.handler)(entry));
            }
        }
        eprintln!("error: command {:?} not found", args[0]);
    }

    pub fn print(&self, t: &Term) {
        self.runner.ui.print(&self.runner.scope, t);
    }

    pub fn format_value(&self, t: &Term) -> String {
        self.runner.ui.format_value(&self.runner.scope, t)
    }

    pub fn spawn(tasks: &[&str]) {
        let mut repl = Repl::default();
        tasks.iter().for_each(|t| repl.parse(t));
    }
}

#[cfg(test)]
pub mod tests {
    use crate::repl::Repl;

    #[test]
    pub fn logic() {
        Repl::spawn(&[":load tests/logic.ac"])
    }

    #[test]
    pub fn tabulation() {
        Repl::spawn(&[":load tests/tabs.ac"]);
        Repl::spawn(&[
            ":load assets/nat.ac",
            ":load assets/combs.ac",
            "Fibo = ^n.(
    If (IsZero (Pred n)) 
        1 
        (Add 
            (Fibo (Pred n))
            (Fibo (Pred (Pred n)))
        )
    )",
            ":gen_nats 0 4",
            ":assert_eq (Fibo 3) 3",
        ]);
    }

    #[test]
    pub fn whitespaced_filepath() {
        Repl::spawn(&[":load \"tests/white spaced.ac\"", ":assert_eq Dark Reasons"])
    }
}
