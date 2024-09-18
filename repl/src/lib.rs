use church::{Body, Term};
use color_eyre::eyre::Result;
use command::Command;
use front::{
    cu::CodeUnit,
    parser::{ParserToken, Token},
    Ast,
};
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
        let tks: Vec<_> = self.cu.into_tokens(src)?.collect();
        if self.settings.show_tokens {
            self.show_tokens(&tks);
        }
        if Self::needs_program_parser(&mut tks.iter()) {
            let ast = self
                .cu
                .program_parser
                .parse(tks)
                .map_err(front::Error::ParserError)?;
            self.cu.eval(ast)?;
        } else {
            let parsed = front::grammar::ExprParser::new()
                .parse(tks)
                .map_err(front::Error::ParserError)?;
            self.reduce_expr(&parsed);
        }
        Ok(())
    }

    fn show_tokens(&mut self, tks: &[ParserToken]) {
        fn str_n_size(n: usize) -> usize {
            (n as f32).log10() as usize + 1
        }

        let end = tks.last().unwrap().2;
        let sp_size = str_n_size(end);
        for (start, tk, end) in tks {
            println!(
                "{start}{} .. {}{end} {tk:?}",
                " ".repeat(sp_size - str_n_size(*start)),
                " ".repeat(sp_size - str_n_size(*end))
            );
        }
    }

    fn reduce_expr(&mut self, ut: &Ast) {
        let mut t = self.cu.scope.dump(ut).unwrap();
        println!("{t}");
        while !self.redex_step(&mut t) {
            self.print_term(&t);
        }
    }

    // Looks like a shitty function, but as the language evolves, it's going to be worth
    fn needs_program_parser<'a>(tokens: &mut impl Iterator<Item = &'a ParserToken>) -> bool {
        tokens.any(|tk| matches!(tk.1, Token::Assign | Token::UseKw))
    }

    pub fn print_term(&mut self, t: &Term) {
        if self.settings.prettify {
            println!("{}", self.cu.scope.pretty_show(t));
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
}
