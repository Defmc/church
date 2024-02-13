use church::{scope::Scope, Body, Term, VarId};
use rustc_hash::FxHashSet as HashSet;
use rustyline::{config::Configurer, error::ReadlineError, DefaultEditor};
use std::{path::PathBuf, str::FromStr};

use crate::repl::cmds::CmdEntry;

use self::mode::Mode;

pub mod cmds;
pub mod env;
pub mod io;
pub mod mode;
pub mod parser;
pub mod proc;

pub type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct Repl {
    scope: Scope,
    loaded_files: HashSet<PathBuf>,
    prompt: String,
    readable: bool,
    mode: Mode,
    quit: bool,
    rl: DefaultEditor,
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
            mode: Mode::default(),
            quit: false,
            prompt: String::from("λ> "),
            rl,
        }
    }
}

impl Repl {
    pub fn start(&mut self) -> Result {
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
        } else if input.contains('=') {
            self.alias(input);
        } else if !input.is_empty() && !input.starts_with('#') {
            self.run(input);
        }
    }

    pub fn run(&mut self, input: &str) {
        let mode = self.mode;
        mode.bench("total", || mode.run(self, input.to_string()));
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
                let mode = self.mode;
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

    pub fn natural_from_church_encoding(s: &Term) -> Option<usize> {
        fn get_natural(f: VarId, x: VarId, s: &Term) -> Option<usize> {
            if let Body::App(lhs, rhs) = s.body.as_ref() {
                if *lhs.body == Body::Id(f) {
                    return get_natural(f, x, rhs).map(|n| n + 1);
                }
            } else if let Body::Id(v) = s.body.as_ref() {
                return (*v == x).then_some(0);
            }

            None
        }

        if let Body::Abs(f, l) = s.body.as_ref() {
            if let Body::Abs(x, l) = l.body.as_ref() {
                return get_natural(*f, *x, l);
            }
            if *l.body == Body::Id(*f) {
                // λf.(λx.(f x))
                // λf.(f) # eta-reduced version of 1
                return Some(1);
            }
        }
        None
    }

    pub fn print_value(&self, b: &Term) {
        println!("{}", self.format_value(b));
    }

    pub fn format_value(&self, b: &Term) -> String {
        if self.readable {
            if let Some(alias) = self.scope.get_from_alpha_key(b) {
                return alias.to_string();
            }
            if let Some(n) = Repl::natural_from_church_encoding(b) {
                return n.to_string();
            }
            if let Some(v) = self.from_list(b) {
                return format!("[{v}]");
            }
            return match b.body.as_ref() {
                Body::Id(id) => church::id_to_str(*id),
                Body::App(ref f, ref x) => format!(
                    "{} {}",
                    self.format_value(f),
                    if usize::from(x.len()) > 1 {
                        format!("({})", self.format_value(x))
                    } else {
                        self.format_value(x)
                    }
                ),
                Body::Abs(v, l) => format!("λ{}.({})", church::id_to_str(*v), self.format_value(l)),
            };
        }
        format!("{b}")
    }

    pub fn from_list(&self, b: &Term) -> Option<String> {
        if let Body::Abs(wrapper, b) = b.body.as_ref() {
            if let Body::App(b, rhs) = b.body.as_ref() {
                if let Body::App(wrap, lhs) = b.body.as_ref() {
                    if &Body::Id(*wrapper) == wrap.body.as_ref() {
                        let mut v = self.format_value(lhs);
                        if let Some(tail) = self.from_list(rhs) {
                            v = format!("{v}, {tail}")
                        } else {
                            v = format!("{v}, {}", self.format_value(rhs))
                        }
                        return Some(v);
                    }
                }
            }
        }
        None
    }

    pub fn print_closed(expr: &Term) {
        println!("{expr}: {} ({:?})", expr.closed, expr.free_variables());
        match expr.body.as_ref() {
            Body::Id(..) => (),
            Body::App(ref lhs, ref rhs) => {
                Self::print_closed(lhs);
                Self::print_closed(rhs);
            }
            Body::Abs(_, ref abs) => Self::print_closed(abs),
        }
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
        println!("hey");
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
}
