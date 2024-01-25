use church::{scope::Scope, Body, VarId};
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
    ("gen_nats ", Repl::gen_nats),
    ("quit", Repl::quit),
];

#[derive(Debug)]
pub struct Repl {
    scope: Scope,
    loaded_files: Vec<PathBuf>,
    prompt: String,
    readable: bool,
    mode: Mode,
    quit: bool,
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
        let mut steps = 0;
        'redex: while l.beta_redex_step() {
            println!("{l}");
            if self == &Self::Debug {
                loop {
                    print!("[step {steps}] (c)ontinue or (a)bort: ");
                    steps += 1;
                    assert!(std::io::stdout().flush().is_ok());
                    buf.clear();
                    assert!(std::io::stdin().read_line(&mut buf).is_ok());
                    match buf.trim() {
                        "c" => break,
                        "a" => break 'redex,
                        "" => break,
                        _ => eprintln!("unknown option"),
                    }
                }
            }
        }
    }

    pub fn run(&self, repl: &Repl, mut l: String) {
        self.bench("delta redex", || {
            l = repl.scope.delta_redex(&l).0;
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
                    });
                    l
                }
                Err(e) => {
                    eprintln!("error: {e:?}");
                    return;
                }
            }
        };
        self.bench("printing", || {
            repl.print_value(&l);
        });
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
            let buf = buf.trim();
            if buf.starts_with(':') {
                self.handle(buf.strip_prefix(':').unwrap())
            } else if buf.contains('=') {
                self.alias(buf);
            } else {
                self.run(buf);
            }
        }
        Ok(())
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
                for (k, v) in self.scope.aliases.iter().zip(self.scope.defs.iter()) {
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
            _ => {
                if let Some(def) = self.scope.indexes.get(input) {
                    println!("{}", self.scope.defs[*def]);
                } else {
                    eprintln!("unknown option {input:?}");
                }
            }
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
        if let Some(value) = input.strip_prefix("readable ") {
            match value {
                "true" => self.readable = true,
                "false" => self.readable = false,
                _ => eprintln!("unknown value {value} for readable"),
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
        let input = self.scope.delta_redex(input).0;
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
        let input = self.scope.delta_redex(input).0;
        let lex = church::parser::lexer(&input);
        match church::parser::parse(lex) {
            Ok(expr) => {
                println!("{}", expr.clone().alpha_reduced());
            }
            Err(e) => eprintln!("error: {e:?}"),
        }
    }

    pub fn delta(&mut self, input: &str) {
        let input = self.scope.delta_redex(input).0;
        let lex = church::parser::lexer(&input);
        match church::parser::parse(lex) {
            Ok(expr) => {
                println!("{}", expr);
            }
            Err(e) => eprintln!("error: {e:?}"),
        }
    }

    pub fn natural_from_church_encoding(s: &Body) -> Option<usize> {
        fn get_natural(f: VarId, x: VarId, s: &Body) -> Option<usize> {
            if let Body::App(lhs, rhs) = s {
                if **lhs == Body::Id(f) {
                    return get_natural(f, x, rhs).map(|n| n + 1);
                }
            } else if let Body::Id(v) = s {
                return (*v == x).then_some(0);
            }

            None
        }

        if let Body::Abs(f, l) = s {
            if let Body::Abs(x, l) = l.as_ref() {
                return get_natural(*f, *x, l);
            } else if *l.as_ref() == Body::Id(*f) {
                // λf.(λx.(f x))
                // λf.(f) # eta-reduced version of 1
                return Some(1);
            }
        }
        None
    }

    pub fn print_value(&self, b: &Body) {
        println!("{}", self.format_value(b));
    }

    pub fn format_value(&self, b: &Body) -> String {
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
        }
        match b {
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
        }
    }

    pub fn from_list(&self, b: &Body) -> Option<String> {
        if let Body::Abs(wrapper, b) = b {
            if let Body::App(b, rhs) = b.as_ref() {
                if let Body::App(wrap, lhs) = b.as_ref() {
                    if Body::Id(*wrapper) == **wrap {
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

    pub fn gen_nats(&mut self, input: &str) {
        if let Some((s, e)) = input.split_once(' ') {
            let s = if let Ok(s) = usize::from_str(s) {
                s
            } else {
                println!("{s:?} is not a valid range start");
                return;
            };
            let e = if let Ok(e) = usize::from_str(e) {
                e
            } else {
                println!("{e:?} is not a valid range end");
                return;
            };
            let mut numbers = Scope::default();
            for i in s..e {
                numbers.aliases.push(i.to_string());
                numbers
                    .defs
                    .push(church::enc::naturals::natural(i).to_string());
            }
            numbers.update();
            self.scope.extend(numbers);
        } else {
            eprintln!("missing two values (start end)");
        }
    }

    pub fn quit(&mut self, _input: &str) {
        self.quit = true;
    }
}
