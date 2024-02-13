use church::{parser::Sym, scope::Scope, Body, Term, VarId};
use logos::Logos;
use rustc_hash::FxHashSet as HashSet;
use rustyline::{config::Configurer, error::ReadlineError, DefaultEditor};
use std::{io::Write, path::PathBuf, str::FromStr, time::Instant};

pub mod env;
pub mod io;
pub mod proc;

pub type Result = std::result::Result<(), Box<dyn std::error::Error>>;

pub const COMMANDS: &[Command] = &[
    Command {
        name: "quit",
        help: "quits the repl",
        inputs_help: &[],
        handler: env::quit_fn,
    },
    Command {
        name: "show",
        help: "shows something from the repl",
        inputs_help: &[("<thing>", "thing to be shown, including:\n\t\t\tscope: all the aliases and expressions defined by the user\n\t\t\tenv: the repl environment\n\t\t\tloaded: all the loaded files as the filepaths\n\t\t\t<alias>: the expression from the scope")],
        handler: env::show_fn,
    },
    Command {
        name: "help",
        help: "show the help message of something",
        inputs_help: &[("<thing>", "thing to be shown")],
        handler: env::help_fn,
    },
    Command {
        name: "set",
        help: "configure something from the repl",
        inputs_help: &[
            (
                "readable <true or false>",
                "sets if the repl should format lambda expressions (default = true)",
            ),
            (
                "prompt <str>",
                "sets the prompt message before each repl line (default = \"λ> \")",
            ),
            ("mode <mode>", "sets the repl mode:\n\t\t\tnormal: it's the default\n\t\t\tdebug: allows you to check and stop every line\n\t\t\tvisual: shows you each step of beta reductions\n\t\t\tbench: benchmarks everything executed"),
            ("history <true or false>", "enable or disable addition to history (default = true)" )
        ],
        handler: env::set,
    },
    Command {
        name: "delta",
        help: "delta reduces the expression"
        ,inputs_help: &[("<expr>", "the lambda expression")],
        handler: proc::delta
    },
    Command {
        name: "assert_eq",
        help: "asserts equality between two lambda expressions. If they're different, panics.",
            inputs_help: &[("<lhs-expr> <rhs-expr>", "the lambda expressions to be compared")],
            handler: env::assert_eq
    },
    Command {
        name: "load",
        help: "runs a file inside repl",
        inputs_help: &[("<filepath>", "file to be run"), ("-s", "strictly load. Updating the lazy-scope immediately")],
        handler: io::load
    },
    Command {
        name: "reload",
        help: "reloads the environment",
        inputs_help: &[("-s", "strictly reload. Updating the lazy-scope after all files have been loaded")]
            ,handler: io::reload
    },
    Command {
        name: "alpha_eq",
        help: "check if two lambda expressions are alpha equivalent",
            inputs_help: &[("<lhs-expr> <rhs-expr>", "the lambda expressions to be compared")],
            handler: proc::alpha_eq
    }
    ,Command {
        name: "alpha",
        help: "alpha reduces the lambda expression",
        inputs_help: &[("<expr>", "the expression to be reduced")],
        handler: proc::alpha
    },
    Command {
        name: "closed",
        help: "show the term's closedness structure",
        inputs_help: &[("<expr>", "the expression to be analyzed")],
        handler: proc::closed
    },
    Command {
        name: "debrejin",
        help: "debrejin-alpha reduces the lambda expression",
        inputs_help: &[("<expr>", "the expression to be reduced")],
        handler: proc::debrejin
    },
    Command {
        name: "prepare",
        help: "immediately updated the scope. Similar to `reload -s`, but way faster (because it doesn't need to load the files again)",
        inputs_help: &[],
        handler: io::prepare
    },
    Command {
        name: "gen_nats",
        help: "binds the natural numbers of the interval",
        inputs_help: &[("<number from> <number to>", "the range of numbers to be binded")],
        handler: env::gen_nats
    },
    Command {
        name: "fix_point",
        help: "solves the recursion of an expression, using the Y combinator",
        inputs_help: &[("<expr>", "the expression to be fixed")],
        handler: proc::fix_point
    },
];

pub struct Command {
    pub name: &'static str,
    pub help: &'static str,
    pub inputs_help: &'static [(&'static str, &'static str)],
    pub handler: fn(CmdEntry),
}

pub struct CmdEntry<'a> {
    pub inputs: Vec<&'a str>,
    pub flags: HashSet<&'a str>,
    pub repl: &'a mut Repl,
}

impl<'a> CmdEntry<'a> {
    pub fn into_expr(&mut self) -> std::result::Result<Term, lrp::Error<Sym>> {
        let input = self.inputs.join(" ");
        let input = self.repl.scope.delta_redex(&input).0;
        Term::try_from_str(input)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord, Logos, Copy)]
pub enum Arg {
    // #[regex(r#""([^\\]|\\.)*""#)]
    #[regex(r#""([^"]|\\.)*""#)]
    StrLit,
    #[regex(r#"[^ ]*"#)]
    Arg,
    #[token("=")]
    Assign,
    #[regex(r"[ \t\n\r]+", logos::skip)]
    Ws,
    #[regex(r#"#.*"#, logos::skip)]
    Comment,
}

impl Arg {
    pub fn parse(s: &impl AsRef<str>) -> impl Iterator<Item = &'_ str> {
        Self::lexer(s.as_ref()).spanned().map(move |(arg, span)| {
            if arg == Ok(Arg::StrLit) {
                &s.as_ref()[span.start + 1..span.end - 1]
            } else {
                &s.as_ref()[span]
            }
        })
    }

    pub fn format(s: &str) -> Option<String> {
        let mut buf = String::with_capacity(s.len());
        let mut it = s.chars();
        while let Some(c) = it.next() {
            if c == '\\' {
                let next = it.next().unwrap();
                let to_push = match next {
                    '0' => '\0',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    _ => return None,
                };
                buf.push(to_push);
            } else {
                buf.push(c);
            }
        }
        Some(buf)
    }
}

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

    pub fn run_show(&self, repl: &mut Repl, l: &mut Term) {
        let mut buf = String::new();
        println!("{l}");
        let mut steps = 0;
        l.update_closed();
        while l.beta_redex_step() {
            println!("{}", repl.format_value(l),);
            if self == &Self::Debug {
                loop {
                    print!("[step {steps}] (c)ontinue or (a)bort: ");
                    steps += 1;
                    assert!(std::io::stdout().flush().is_ok());
                    buf.clear();
                    assert!(std::io::stdin().read_line(&mut buf).is_ok());
                    match buf.trim() {
                        "c" => break,
                        "a" => return,
                        "" => break,
                        _ => eprintln!("unknown option"),
                    }
                }
            }
        }
    }

    pub fn run(&self, repl: &mut Repl, mut l: String) {
        self.bench("delta redex", || {
            l = repl.scope.delta_redex(&l).0;
        });
        let l = if self.should_show() {
            match Term::try_from_str(&l) {
                Ok(mut l) => {
                    self.bench("beta redex", || self.run_show(repl, &mut l));
                    l
                }
                Err(e) => {
                    eprintln!("error: {e:?}");
                    return;
                }
            }
        } else {
            match Term::try_from_str(&l) {
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
            let args: Vec<_> = Arg::parse(&input).collect();
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
        for hs in COMMANDS.iter() {
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
        for task in tasks {
            repl.parse(task);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::repl::Repl;

    #[test]
    pub fn logic() {
        Repl::spawn(&[":load tests/logic.ac"])
    }
}
