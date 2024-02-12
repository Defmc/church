use church::{parser::Sym, scope::Scope, Body, Term, VarId};
use logos::Logos;
use rustc_hash::FxHashSet as HashSet;
use rustyline::{config::Configurer, error::ReadlineError, DefaultEditor};
use std::{fmt, fs::read_to_string, io::Write, path::PathBuf, rc::Rc, str::FromStr, time::Instant};

pub type Result = std::result::Result<(), Box<dyn std::error::Error>>;

pub const COMMANDS: &[Command] = &[
    Command {
        name: "quit",
        help: "quits the repl",
        inputs_help: &[],
        handler: quit_fn,
    },
    Command {
        name: "show",
        help: "shows something from the repl",
        inputs_help: &[("<thing>", "thing to be shown, including:\n\t\t\tscope: all the aliases and expressions defined by the user\n\t\t\tenv: the repl environment\n\t\t\tloaded: all the loaded files as the filepaths\n\t\t\t<alias>: the expression from the scope")],
        handler: show_fn,
    },
    Command {
        name: "help",
        help: "show the help message of something",
        inputs_help: &[("<thing>", "thing to be shown")],
        handler: help_fn,
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
        handler: set,
    },
    Command {
        name: "delta",
        help: "delta reduces the expression"
        ,inputs_help: &[("<expr>", "the lambda expression")],
        handler: delta
    },
    Command {
        name: "assert_eq",
        help: "asserts equality between two lambda expressions. If they're different, panics.",
            inputs_help: &[("<lhs-expr> <rhs-expr>", "the lambda expressions to be compared")],
            handler: assert_eq
    },
    Command {
        name: "load",
        help: "runs a file inside repl",
        inputs_help: &[("<filepath>", "file to be run"), ("-s", "strictly load. Updating the lazy-scope immediately")],
        handler: load
    },
    Command {
        name: "reload",
        help: "reloads the environment",
        inputs_help: &[("-s", "strictly reload. Updating the lazy-scope after all files have been loaded")]
            ,handler: reload
    },
    Command {
        name: "alpha_eq",
        help: "check if two lambda expressions are alpha equivalent",
            inputs_help: &[("<lhs-expr> <rhs-expr>", "the lambda expressions to be compared")],
            handler: alpha_eq
    }
    ,Command {
        name: "alpha",
        help: "alpha reduces the lambda expression",
        inputs_help: &[("<expr>", "the expression to be reduced")],
        handler: alpha
    },
    Command {
        name: "closed",
        help: "show the term's closedness structure",
        inputs_help: &[("<expr>", "the expression to be analyzed")],
        handler: closed
    },
    Command {
        name: "debrejin",
        help: "debrejin-alpha reduces the lambda expression",
        inputs_help: &[("<expr>", "the expression to be reduced")],
        handler: debrejin
    },
    Command {
        name: "prepare",
        help: "immediately updated the scope. Similar to `reload -s`, but way faster (because it doesn't need to load the files again)",
        inputs_help: &[],
        handler: prepare
    },
    Command {
        name: "gen_nats",
        help: "binds the natural numbers of the interval",
        inputs_help: &[("<number from> <number to>", "the range of numbers to be binded")],
        handler: gen_nats
    },
    Command {
        name: "fix_point",
        help: "solves the recursion of an expression, using the Y combinator",
        inputs_help: &[("<expr>", "the expression to be fixed")],
        handler: fix_point
    }
];

fn quit_fn(e: CmdEntry) {
    e.repl.quit = true;
}

fn show_fn(e: CmdEntry) {
    match e.inputs[0] {
        "scope" => {
            for (k, v) in e.repl.scope.aliases.iter().zip(e.repl.scope.defs.iter()) {
                println!("{k} = {v}");
            }
        }
        "env" => {
            println!("{:?}", e.repl);
        }
        "loaded" => {
            for p in e.repl.loaded_files.iter() {
                println!("{p:?}");
            }
        }
        _ => {
            if let Some(def) = e.repl.scope.indexes.get(e.inputs[0]) {
                println!("{}", e.repl.scope.defs[*def]);
            } else {
                eprintln!("unknown option {:?}", e.inputs[0]);
            }
        }
    }
}

fn set(e: CmdEntry) {
    fn set_with<T: FromStr>(opt: &mut T, s: &str)
    where
        <T as FromStr>::Err: fmt::Debug,
    {
        match T::from_str(s) {
            Ok(v) => *opt = v,
            Err(e) => println!("unknown value {:?}: {e:?}", s),
        }
    }

    match e.inputs.len() {
        1 => {
            eprintln!("missing option and value");
            return;
        }
        2 => {
            eprintln!("missing value");
            return;
        }
        _ => (),
    };
    match e.inputs[0] {
        "readable" => set_with(&mut e.repl.readable, e.inputs[1]),
        "prompt" => match Arg::format(e.inputs[1]) {
            Some(v) => set_with(&mut e.repl.prompt, &v),
            None => eprintln!("bad format string {:?}", e.inputs[1]),
        },
        "mode" => set_with(&mut e.repl.mode, e.inputs[1]),
        "history" => match bool::from_str(e.inputs[1]) {
            Ok(opt) => e.repl.rl.set_auto_add_history(opt),
            Err(err) => eprintln!("unknown value {:?}: {err:?}", e.inputs[1]),
        },
        _ => eprintln!("unknonwn option {:?}", e.inputs[0]),
    }
}

fn help_fn(e: CmdEntry) {
    for hs in COMMANDS.iter() {
        if e.inputs[0] == hs.name {
            println!("{}", hs.name);
            println!("\tdescription:");
            println!("\t\t{}", hs.help);
            println!("\tinputs:");
            for (i, h) in hs.inputs_help {
                println!("\t\t{i}: {h}");
            }
        }
    }
}

fn delta(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(expr) => {
            println!("{}", expr);
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
}

fn assert_eq(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(mut expr) => match Rc::make_mut(&mut expr.body) {
            Body::App(ref mut lhs, ref mut rhs) => {
                let (lhs_s, rhs_s) = (e.repl.format_value(lhs), e.repl.format_value(rhs));
                print!("testing {lhs_s} == {rhs_s}... ",);
                std::io::stdout().flush().unwrap();
                lhs.beta_redex();
                rhs.beta_redex();
                if !lhs.alpha_eq(rhs) {
                    eprintln!("\nerror: they're different");
                    eprintln!("\t{lhs_s} -> {}", e.repl.format_value(lhs));
                    eprintln!("\t{rhs_s} -> {}", e.repl.format_value(rhs));
                    panic!("assertion failed");
                }
                println!("ok!");
            }
            _ => eprintln!("error: missing another expression"),
        },
        Err(e) => eprintln!("error: {e:?}"),
    }
}

fn load(e: CmdEntry) {
    let input = e.inputs[0].into();
    if e.repl.loaded_files.contains(&input) {
        eprintln!("warn: already loaded {input:?}");
        return;
    }
    match read_to_string(&input) {
        Ok(s) => {
            s.lines().for_each(|l| e.repl.parse(l));
            e.repl.loaded_files.insert(input);
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
    if e.flags.contains(&"-s") {
        e.repl.scope.update();
    }
}

fn reload(e: CmdEntry) {
    e.repl.scope = Scope::default();
    let loaded = e.repl.loaded_files.clone();
    e.repl.loaded_files.clear();
    loaded.into_iter().for_each(|f| {
        load(CmdEntry {
            inputs: vec![&f.to_string_lossy()],
            flags: HashSet::default(),
            repl: e.repl,
        })
    });

    if e.flags.contains(&"-s") {
        e.repl.scope.update();
    }
}

fn alpha_eq(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(expr) => match expr.body.as_ref() {
            Body::App(ref lhs, ref rhs) => {
                println!("{}", lhs.alpha_eq(rhs));
            }
            _ => eprintln!("missing the second expression"),
        },
        Err(e) => eprintln!("error: {e:?}"),
    }
}
fn alpha(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(expr) => {
            println!("{}", expr.alpha_reduced());
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
}

fn closed(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(expr) => {
            e.repl.print_closed(&expr);
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
}

fn debrejin(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(l) => {
            println!("{}", l.clone().debrejin_reduced());
            e.repl.mode.bench("printing", || {
                e.repl.print_value(&l);
            });
        }
        Err(e) => {
            eprintln!("error: {e:?}");
        }
    }
}

fn prepare(e: CmdEntry) {
    e.repl.scope.update();
}

fn gen_nats(e: CmdEntry) {
    fn natural(n: usize) -> Term {
        fn natural_body(n: usize) -> Term {
            let body = if n == 0 {
                Body::Id(1)
            } else {
                Body::App(Term::new(Body::Id(0)), natural_body(n - 1))
            };
            Term::new(body)
        }
        natural_body(n).with([0, 1])
    }
    let start = if let Ok(s) = usize::from_str(e.inputs[0]) {
        s
    } else {
        println!("{:?} is not a valid range start", e.inputs[0]);
        return;
    };
    let end = if let Ok(e) = usize::from_str(e.inputs[1]) {
        e
    } else {
        println!("{:?} is not a valid range end", e.inputs[1]);
        return;
    };
    let mut numbers = Scope::default();
    for i in start..end {
        numbers.aliases.push(i.to_string());
        numbers.defs.push(natural(i).to_string());
    }
    numbers.update();
    e.repl.scope.extend(numbers);
}

fn fix_point(e: CmdEntry) {
    e.repl
        .mode
        .bench("fix point", || match Scope::from_str(&e.inputs.join(" ")) {
            Ok(s) => {
                Scope::solve_recursion(&s.aliases[0], &s.defs[0]).map_or_else(
                    || println!("{} = {}", s.aliases[0], s.defs[0]),
                    |imp| println!("{} = {imp}", s.aliases[0]),
                );
            }
            Err(e) => eprintln!("error while parsing scope: {e:?}"),
        })
}

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
                let mode = self.mode.clone();
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
                        .filter_map(|s| {
                            if s.starts_with('-') {
                                Some(s[1..].into())
                            } else {
                                None
                            }
                        })
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

    pub fn print_closed(&mut self, expr: &Term) {
        println!("{expr}: {} ({:?})", expr.closed, expr.free_variables());
        match expr.body.as_ref() {
            Body::Id(..) => (),
            Body::App(ref lhs, ref rhs) => {
                self.print_closed(lhs);
                self.print_closed(rhs);
            }
            Body::Abs(_, ref abs) => self.print_closed(abs),
        }
    }
}
