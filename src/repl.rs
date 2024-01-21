use church::{scope::Scope, Body};
use rustyline::{config::Configurer, error::ReadlineError, DefaultEditor};
use std::{fs::read_to_string, path::PathBuf, str::FromStr};

pub type Result = std::result::Result<(), Box<dyn std::error::Error>>;

pub type Handler = fn(&mut Repl, &str);

pub const HANDLERS: &[(&str, Handler)] = &[
    ("show ", Repl::show),
    ("load ", Repl::load),
    ("set ", Repl::set),
    ("alpha_eq ", Repl::alpha_eq),
    ("alpha ", Repl::alpha),
    ("delta", Repl::delta),
];

#[derive(Debug)]
pub struct Repl {
    scope: Scope,
    loaded_files: Vec<PathBuf>,
    prompt: String,
    show_alias: bool,
    trace: bool,
    rl: DefaultEditor,
}

impl Default for Repl {
    fn default() -> Self {
        let mut rl = DefaultEditor::new().unwrap();
        rl.set_auto_add_history(true);
        rl.set_history_ignore_space(true);
        Repl {
            scope: Scope::default(),
            loaded_files: Vec::default(),
            show_alias: true,
            trace: false,
            prompt: String::from("λ> "),
            rl,
        }
    }
}

impl Repl {
    pub fn start(&mut self) -> Result {
        loop {
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
            let buf = buf.trim();
            if buf.starts_with(':') {
                self.handle(buf.strip_prefix(':').unwrap())
            } else if buf.contains('=') {
                self.alias(buf);
            } else {
                self.run(buf);
            }
        }
    }

    pub fn run(&mut self, input: &str) {
        let mut input = input.to_string();
        self.scope.delta_redex(&mut input);
        let lex = church::parser::lexer(&input);
        match church::parser::parse(lex) {
            Ok(expr) => {
                let normal = if self.trace {
                    let mut expr = expr.clone();
                    println!("{expr}");
                    while expr.beta_redex() != false {
                        println!("{expr}");
                    }
                    expr
                } else {
                    expr.clone().beta_reduced()
                };
                if self.show_alias {
                    if let Some(k) = self.scope.get_from_alpha_key(&normal) {
                        println!("{k}");
                        return;
                    }
                }
                println!("{normal}");
            }
            Err(e) => eprintln!("error: {e:?}"),
        }
    }

    pub fn alias(&mut self, input: &str) {
        match Scope::from_str(input) {
            Ok(nscope) => self.scope.extend(nscope),
            Err(e) => eprintln!("error: {e:?}"),
        }
    }

    pub fn handle(&mut self, input: &str) {
        for (prefix, h) in HANDLERS.iter() {
            if input.starts_with(prefix) {
                let stripped = input.strip_prefix(prefix).unwrap();
                return h(self, stripped);
            }
        }
        eprintln!("error: command {input:?} not found");
    }

    pub fn show(&mut self, input: &str) {
        match input {
            "scope" => {
                for (k, v) in self.scope.defs.iter() {
                    println!("{k} = {v}");
                }
            }
            "env" => {
                println!("{self:?}");
            }
            "loaded" => {
                for p in self.loaded_files.iter() {
                    println!("{p:?}");
                }
            }
            _ if self.scope.defs.contains_key(input) => {
                println!("{}", self.scope.defs[input])
            }
            _ => eprintln!("unknown option {input:?}"),
        }
    }

    pub fn load(&mut self, input: &str) {
        match read_to_string(input) {
            Ok(s) => {
                self.alias(&s);
                self.loaded_files.push(input.into());
            }
            Err(e) => eprintln!("error: {e:?}"),
        }
    }

    pub fn set(&mut self, input: &str) {
        if let Some(value) = input.strip_prefix("show_alias ") {
            match value {
                "true" => self.show_alias = true,
                "false" => self.show_alias = false,
                _ => eprintln!("unknown value {value} for show_alias"),
            }
        } else if let Some(value) = input.strip_prefix("prompt ") {
            self.prompt = value.to_string();
        } else if let Some(value) = input.strip_prefix("trace ") {
            match value {
                "true" => self.trace = true,
                "false" => self.trace = false,
                _ => eprintln!("unknown value {value} for trace"),
            }
        } else {
            eprintln!("unknown option {input}");
        }
    }

    pub fn alpha_eq(&mut self, input: &str) {
        let mut input = input.to_string();
        self.scope.delta_redex(&mut input);
        let lex = church::parser::lexer(&input);
        match church::parser::parse(lex) {
            Ok(expr) => match expr {
                Body::App(ref lhs, ref rhs) => {
                    println!("{}", if lhs.alpha_eq(rhs) { "true" } else { "false" });
                }
                _ => eprintln!("missing the second expression"),
            },
            Err(e) => eprintln!("error: {e:?}"),
        }
    }

    pub fn alpha(&mut self, input: &str) {
        let mut input = input.to_string();
        self.scope.delta_redex(&mut input);
        let lex = church::parser::lexer(&input);
        match church::parser::parse(lex) {
            Ok(expr) => {
                println!("{}", expr.clone().alpha_reduced());
            }
            Err(e) => eprintln!("error: {e:?}"),
        }
    }
    pub fn delta(&mut self, input: &str) {
        let mut input = input.to_string();
        self.scope.delta_redex(&mut input);
        let lex = church::parser::lexer(&input);
        match church::parser::parse(lex) {
            Ok(expr) => {
                println!("{}", expr);
            }
            Err(e) => eprintln!("error: {e:?}"),
        }
    }
}
