use color_eyre::eyre::Result;
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
            self.eval(input)
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
        println!("{src} -> {p}");
        while !p.normal_beta_redex_step() {
            println!("{p}");
        }
        Ok(())
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
}
