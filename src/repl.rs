use church::{scope::Scope, Body};
use rustyline::{config::Configurer, error::ReadlineError, DefaultEditor};
use std::{fs::read_to_string, io::Write, path::PathBuf, str::FromStr, time::Instant};

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
    mode: Mode,
    rl: DefaultEditor,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Debug,
    Visual,
    Bench,
    #[default]
    Normal,
}

impl FromStr for Mode {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "debug" => Ok(Self::Debug),
            "visual" => Ok(Self::Visual),
            "normal" => Ok(Self::Normal),
            "bench" => Ok(Self::Bench),
            _ => Err(()),
        }
    }
}

impl Mode {
    pub fn should_show(&self) -> bool {
        self == &Self::Visual || self == &Self::Debug
    }

    pub fn bench(&self, op: &str, f: impl FnOnce()) {
        let start = Instant::now();
        f();
        if self == &Self::Bench {
            println!("[{op}: {:?}]", start.elapsed())
        };
    }

    pub fn run_show(&self, l: &mut Body) {
        let mut buf = String::new();
        println!("{l}");
        'redex: while l.beta_redex() {
            println!("{l}");
            if self == &Self::Debug {
                loop {
                    print!("(c)ontinue or (a)bort: ");
                    assert!(std::io::stdout().flush().is_ok());
                    buf.clear();
                    assert!(std::io::stdin().read_line(&mut buf).is_ok());
                    match buf.trim() {
                        "c" => break,
                        "a" => break 'redex,
                        _ => eprintln!("unknown option"),
                    }
                }
            }
        }
    }

    pub fn run(&self, repl: &Repl, mut l: String) {
        self.bench("delta redex", || {
            repl.scope.delta_redex(&mut l);
        });
        let l = if self.should_show() {
            match Body::from_str(&l) {
                Ok(mut l) => {
                    self.bench("beta redex", || self.run_show(&mut l));
                    l
                }
                Err(e) => {
                    eprintln!("error: {e:?}");
                    return;
                }
            }
        } else {
            match Body::from_str(&l) {
                Ok(mut l) => {
                    self.bench("beta redex", || {
                        l.beta_redex();
                        println!("{l}");
                    });
                    l
                }
                Err(e) => {
                    eprintln!("error: {e:?}");
                    return;
                }
            }
        };
        if repl.show_alias {
            self.bench("alias matching", || {
                if let Some(alias) = repl.scope.get_from_alpha_key(&l) {
                    println!("{alias}");
                }
            });
        }
    }
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
            mode: Mode::default(),
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
        self.mode
            .bench("total", || self.mode.run(self, input.to_string()));
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
                return self.mode.clone().bench(prefix, || h(self, stripped));
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
        } else if let Some(value) = input.strip_prefix("mode ") {
            if let Ok(value) = Mode::from_str(value) {
                self.mode = value;
            } else {
                eprintln!("unknown value {value} for mode");
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
