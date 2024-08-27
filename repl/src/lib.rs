use church::{Body, Term};
use color_eyre::eyre::{eyre, Result};
use command::Command;
use front::scope::Scope;
use rustyline::{error::ReadlineError, DefaultEditor};
use settings::Settings;
use std::{collections::HashMap, time::Instant};

pub use args::Err;

pub mod args;
pub mod command;
pub mod settings;

pub struct Repl {
    pub scope: Scope,
    pub rl: DefaultEditor,
    pub settings: Settings,
    pub commands: HashMap<String, Command>,
    pub should_exit: bool,
}

impl Default for Repl {
    fn default() -> Self {
        Self {
            scope: Scope::default(),
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
        let mut p = self.scope.into_term(src)?;
        if self.settings.show_ast {
            Self::show_ast(&p, 0);
        }
        println!("{src} -> {p}");
        while !p.normal_beta_redex_step() {

        while !self.redex_step(&mut p) {
            if self.settings.prettify {
                println!("{}", self.scope.pretty_show(&p));
            } else {
                println!("{p}");
            }
        }
        Ok(())
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
        let m = self.scope.into_term(params[1])?;
        self.scope.insert(var.to_string(), m)?;
        Ok(())
    }
}
