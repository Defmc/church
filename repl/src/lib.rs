use church::{Body, Term};
use color_eyre::eyre::{eyre, Result};
use command::Command;
use front::cu::CodeUnit;
use rustyline::{error::ReadlineError, DefaultEditor};
use settings::Settings;
use std::{collections::HashMap, time::Instant};

pub use args::Err;

pub mod args;
pub mod command;
pub mod err;
pub mod settings;
pub use err::Error;

pub struct Repl {
    pub cu: CodeUnit,
    pub rl: DefaultEditor,
    pub settings: Settings,
    pub commands: HashMap<String, Command>,
    pub should_exit: bool,
}

impl Default for Repl {
    fn default() -> Self {
        Self {
            cu: CodeUnit::default(),
            rl: DefaultEditor::new().unwrap(),
            settings: Settings::default(),
            commands: command::COMMANDS
                .iter()
                .cloned()
                .map(|c| (c.name.to_owned(), c))
                .collect(),
            should_exit: false,
        }
    }
}

impl Repl {
    pub fn run(&mut self) -> Result<()> {
        while !self.should_exit {
            match self.rl.readline(&self.settings.prompt) {
                Ok(l) => self.handle(&l),
                Err(ReadlineError::Interrupted) => {}
                Err(ReadlineError::Eof) => break,
                Err(e) => {
                    eprintln!("error: {e:?}")
                }
            }
        }
        Ok(())
    }

    pub fn handle(&mut self, input: &str) {
        assert!(self.rl.add_history_entry(input).is_ok());
        if let Some(s) = input.strip_prefix(":") {
            self.cmd(s)
        } else {
            if input.contains('=') {
                self.set(input)
            } else {
                self.eval(input)
            }
        }
        .unwrap_or_else(|e| eprintln!("err: {e:?}"))
    }

    pub fn cmd(&mut self, s: &str) -> Result<()> {
        let args = args::get_args(s).ok_or_else(|| Err::ValueParserError(s.into()))?;
        if let Some(cmd) = self.commands.get(&args[0]) {
            if args.len() - 1 != cmd.args.len() {
                Err(Err::MissingArgs(args.len() - 1, cmd.args.len()))?;
            }
            return (cmd.cmd)(self, &args[1..]);
        } else {
            Err(Err::UnknownCommand(args[0].to_owned()))?;
        }
        unreachable!()
    }

    pub fn eval(&mut self, src: &str) -> Result<()> {
        let iter = self.cu.into_iter(src);
        let parsed = self.cu.atom_parser.parse(iter).unwrap();
        if let Some(mut p) = self.cu.eval(parsed).unwrap() {
            println!("{p}");
            while !self.redex_step(&mut p) {
                self.print_term(&p);
            }
        }
        Ok(())
    }

    pub fn print_term(&mut self, t: &Term) {
        if self.settings.prettify {
            println!("{}", self.cu.scope.pretty_show(&t));
        } else {
            println!("{t}");
        }
    }

    fn redex_step(&self, p: &mut Term) -> bool {
        match self.settings.b_order {
            settings::BetaOrder::Normal => p.normal_beta_redex_step(),
            settings::BetaOrder::CallByValue => p.cbv_beta_redex_step(),
        }
    }

    pub fn show_ast(p: &Term, depth: usize) {
        let tab = "\t".repeat(depth);
        print!("{tab}");
        match p.body.as_ref() {
            Body::Var(v) => println!("var {v}"),
            Body::App(m, n) => {
                println!("app:");
                Self::show_ast(m, depth + 1);
                Self::show_ast(n, depth + 1);
            }
            Body::Abs(v, m) => {
                println!("abs {v}:");
                Self::show_ast(m, depth + 1);
            }
        }
    }

    pub fn bench<T>(&mut self, task: &str, f: impl FnOnce(&mut Self) -> T) -> T {
        let start = Instant::now();
        let v = f(self);
        let elapsed = start.elapsed();
        if self.settings.bench {
            println!("time {task}: {elapsed:?}");
        }
        v
    }

    pub fn set(&mut self, s: &str) -> Result<()> {
        let params: Vec<_> = s.split('=').collect();
        if params.len() != 2 {
            return Err(eyre!("{s} should be just `var = M`"));
        }
        let var = front::parser::try_from_str(&params[0]).unwrap();
        let var = if let front::UBody::Var(s) = *var.body {
            s
        } else {
            return Err(eyre!("{s} should be just `var = M`"));
        };
        let m = self.cu.scope.into_term(params[1])?;
        self.cu.scope.insert(var.to_string(), m)?;
        Ok(())
    }
}
